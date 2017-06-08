// Lumol, an extensible molecular simulation engine
// Copyright (C) Lumol's contributors — BSD license

use std::io::prelude::*;
use std::io;
use std::error;
use std::fmt;
use std::fs::File;
use std::path::{Path, PathBuf};

use caldyn::{Expr, Context};
use caldyn::Error as CaldynError;

use super::Output;
use sys::System;
use units;

/// Possible causes of error when using a custom output
#[derive(Debug)]
pub enum CustomOutputError {
    /// Any IO error
    Io(io::Error),
    /// Error in the mathematical expression
    Expr(CaldynError),
    /// Any other error
    Custom(String),
}

impl From<io::Error> for CustomOutputError {
    fn from(error: io::Error) -> CustomOutputError {
        CustomOutputError::Io(error)
    }
}

impl From<CaldynError> for CustomOutputError {
    fn from(error: CaldynError) -> CustomOutputError {
        CustomOutputError::Expr(error)
    }
}

impl From<String> for CustomOutputError {
    fn from(error: String) -> CustomOutputError {
        CustomOutputError::Custom(error)
    }
}

impl fmt::Display for CustomOutputError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            CustomOutputError::Io(ref err) => try!(write!(fmt, "{}", err)),
            CustomOutputError::Expr(ref err) => try!(write!(fmt, "{}", err)),
            CustomOutputError::Custom(ref err) => try!(write!(fmt, "{}", err)),
        }
        Ok(())
    }
}

impl error::Error for CustomOutputError {
    fn description(&self) -> &str {
        match *self {
            CustomOutputError::Io(ref err) => err.description(),
            CustomOutputError::Expr(ref err) => err.description(),
            CustomOutputError::Custom(ref err) => err,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            CustomOutputError::Io(ref err) => Some(err),
            CustomOutputError::Expr(ref err) => Some(err),
            CustomOutputError::Custom(_) => None,
        }
    }
}

/// Helper struct to parse and format custom output strings
struct FormatArgs {
    /// Pairs of "constant string", "format expression"
    args: Vec<(String, Expr)>,
    /// Any remaining tail after the last expression
    tail: String
}

impl FormatArgs {
    fn new(format: &str) -> Result<FormatArgs, CustomOutputError> {
        let mut args = Vec::new();
        let mut expr = String::new();
        let mut tail = String::new();

        let mut in_expr = false;
        for c in format.chars() {
            match c {
                '{' if !in_expr => {
                    in_expr = true;
                }
                '}' if in_expr => {
                    in_expr = false;
                    let ex = try!(Expr::parse(&expr));
                    args.push((tail.clone(), ex));
                    tail.clear();
                    expr.clear();
                }
                '{' if in_expr => {
                    return Err(CustomOutputError::Custom(
                        "found { in an expression".into()
                    ));
                }
                '}' if !in_expr => {
                    return Err(CustomOutputError::Custom(
                        "found } outside of an expression".into()
                    ));
                }
                c => {
                    if in_expr {
                        expr.push(c);
                    } else {
                        tail.push(c);
                    }
                }
            }
        }
        if in_expr {
            return Err(CustomOutputError::Custom(
                "mismatched braces".into()
            ));
        }

        Ok(FormatArgs {
            args: args,
            tail: tail,
        })
    }

    fn get_context<'a>(&self, system: &'a System) -> Context<'a> {
        let mut context = Context::new();
        context.set_query(move |name| {
            // Get unit conversion factor firsts
            units::FACTORS.get(name).cloned().or_else(|| {
                macro_rules! get_particle_data {
                    ($index: ident, $callback: expr) => (
                        system.particles()
                              .nth($index)
                              .map($callback)
                              .unwrap_or_else(|| {
                                  warn_once!(
                                      "index out of bound in custom output: \
                                      index is {}, but we only have {} atoms",
                                      $index, system.size()
                                  );
                                  return 0.0;
                              })
                    );
                }
                if name.contains('[') {
                    // vector data
                    let (name, index) = parse_index(name);
                    match name {
                        // position
                        "x" => Some(get_particle_data!(index, |p| p.position[0])),
                        "y" => Some(get_particle_data!(index, |p| p.position[1])),
                        "z" => Some(get_particle_data!(index, |p| p.position[2])),
                        // velocity
                        "vx" => Some(get_particle_data!(index, |p| p.velocity[0])),
                        "vy" => Some(get_particle_data!(index, |p| p.velocity[1])),
                        "vz" => Some(get_particle_data!(index, |p| p.velocity[2])),
                        // other atomic properties
                        "mass" => Some(get_particle_data!(index, |p| p.mass)),
                        "charge" => Some(get_particle_data!(index, |p| p.charge)),
                        _ => None
                    }
                } else {
                    // scalar data
                    match name {
                        "pressure" => Some(system.pressure()),
                        "volume" => Some(system.volume()),
                        "temperature" => Some(system.temperature()),
                        "natoms" => Some(system.size() as f64),
                        "cell.a" => Some(system.cell.a()),
                        "cell.b" => Some(system.cell.b()),
                        "cell.c" => Some(system.cell.c()),
                        "cell.alpha" => Some(system.cell.alpha()),
                        "cell.beta" => Some(system.cell.beta()),
                        "cell.gamma" => Some(system.cell.gamma()),
                        _ => None
                    }
                }
            })
        });

        return context;
    }

    fn format(&self, system: &System) -> Result<String, CustomOutputError> {
        let context = self.get_context(system);
        let mut output = String::new();
        for &(ref string, ref expr) in &self.args {
            output.push_str(string);
            let value = try!(expr.eval(&context));
            output.push_str(&value.to_string());
        }
        output.push_str(&self.tail);
        return Ok(output);
    }
}

/// Get the name and index in a string looking like `name[index]`. Everything
/// else is just passed through.
fn parse_index(input: &str) -> (&str, usize) {
    // We can index `input`, because caldyn only works with ASCII data
    let lbrackets = input.match_indices('[').collect::<Vec<_>>();
    let rbrackets = input.match_indices(']').collect::<Vec<_>>();

    if lbrackets.len() != 1 || rbrackets.len() != 1 {
        // More than one bracket
        return (input, 0);
    }

    let start = lbrackets[0].0;
    let end = rbrackets[0].0;
    if start > end {
        // `[` is after `]`
        return (input, 0);
    }

    if let Ok(index) = input[(start + 1)..end].parse() {
        return (&input[..start], index);
    } else {
        // invalid integer value
        return (input, 0);
    }
}

/// The `CustomOutput` write data into a file from an user-provided template.
///
/// The template string can contain mathematical expressions, using some
/// physical properties of the system. These mathematical expressions must be
/// enclosed in braces (`{}`). Here are some examples:
///
/// - A constant string is reproduces as it is: `some data`;
/// - Anything in braces is replaced by the corresponding values: `{pressure} {volume}`;
/// - Mathematical operators are allowed in braces: `{pressure / volume}`. You
///   can use `+`, `-`, `/`, `*`, `^` for exponentiation and parentheses;
/// - Some properties are arrays of atomic properties `{x[0] + y[20]}`;
/// - Finally, all the properties are given in the internal units. One can
///   specify another unit: `x[0] / nm`.
///
/// Here is a list of all accepted properties:
///
/// - Atomic properties: `x`, `y` and `z` for cartesian coordinates, `vx`, `vy`
///   and `vz` for cartesian components of the velocity , `mass` for the atomic
///   mass, `charge` for the atomic charge.
/// - Physical properties: `pressure`, `volume`, `temperature`, `natoms`
/// - Unit Cell properties: `cell.a`, `cell.b`, `cell.c` are the unit cell
///   vector lengths; `cell.alpha`, `cell.beta` and `cell.gamma` are the unit
///   cell angles.
pub struct CustomOutput {
    file: File,
    path: PathBuf,
    template: String,
    args: FormatArgs,
}

impl CustomOutput {
    /// Create a new `CustomOutput` writing to the file at `filename` using
    /// the given `template`. The `template` is only partially validated at
    /// this stage.
    pub fn new<P: AsRef<Path>>(filename: P, template: &str) -> Result<CustomOutput, CustomOutputError> {
        Ok(CustomOutput {
            file: try!(File::create(filename.as_ref())),
            path: filename.as_ref().to_owned(),
            template: template.into(),
            args: try!(FormatArgs::new(template)),
        })
    }
}

impl Output for CustomOutput {
    fn setup(&mut self, _: &System) {
        if let Err(err) = writeln!(&mut self.file, "# Custom output") {
            fatal_error!("Could not write to file '{}': {}", self.path.display(), err);
        }
        if let Err(err) = writeln!(&mut self.file, "# {}", self.template) {
            fatal_error!("Could not write to file '{}': {}", self.path.display(), err);
        }
    }

    fn write(&mut self, system: &System) {
        if let Ok(formatted) = self.args.format(system) {
            if let Err(err) = writeln!(&mut self.file, "{}", formatted) {
                error!("Could not write to file '{}': {}", self.path.display(), err);
            }
        } else {
            error_once!("Could not evaluate custom output {}", self.template);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tests::{test_output, testing_system};

    fn format(input: &str) -> String {
        FormatArgs::new(input).unwrap().format(&testing_system()).unwrap()
    }

    #[test]
    fn parsing_index() {
        assert_eq!(parse_index("a[6]"), ("a", 6));

        assert_eq!(parse_index("a"), ("a", 0));
        assert_eq!(parse_index("a][6"), ("a][6", 0));
        assert_eq!(parse_index("a[6][2]"), ("a[6][2]", 0));
        assert_eq!(parse_index("a[6]2]"), ("a[6]2]", 0));
        assert_eq!(parse_index("a[6][2"), ("a[6][2", 0));
        assert_eq!(parse_index("a[b]"), ("a[b]", 0));
    }

    #[test]
    fn format_args_parsing() {
        assert!(FormatArgs::new("one {test} two {5 } three!").is_ok());

        assert!(FormatArgs::new("{3 + 4} {").is_err());
        assert!(FormatArgs::new("{3 + 4} }").is_err());
        assert!(FormatArgs::new("{3 + { 4}").is_err());
        assert!(FormatArgs::new("{3 + {} }").is_err());
    }

    #[test]
    fn formating() {
        assert_eq!(format("{3 + 4}"), "7");

        assert_eq!(format("{pressure / bar}"), "10299.991728079816");
        assert_eq!(format("{temperature / K}"), "38083.04389172312");
        assert_eq!(format("{volume / A^3}"), "1000");

        assert_eq!(format("{cell.a / A}"), "10");
        assert_eq!(format("{cell.b / A}"), "10");
        assert_eq!(format("{cell.c / A}"), "10");
        assert_eq!(format("{cell.alpha}"), "90");
        assert_eq!(format("{cell.beta}"),  "90");
        assert_eq!(format("{cell.gamma}"), "90");

        assert_eq!(format("{x[1]}"), "1.3");
        assert_eq!(format("{vy[1]}"), "0");
        assert_eq!(format("{vx[0]}"), "0.1");

        assert_eq!(format("{cell.a / bohr}"), "18.897261328856434");
        assert_eq!(format("{cell.a / nm}"), "1");
        assert_eq!(format("{cell.a / m}"), "0.000000001");
    }

    #[test]
    fn custom() {
        test_output(|path| {
            Box::new(CustomOutput::new(path, "p {pressure/bar} t {3 * 5} \tff").unwrap())
        },
"# Custom output
# p {pressure/bar} t {3 * 5} \tff
p 10299.991728079816 t 15 \tff
"
        );
    }
}
