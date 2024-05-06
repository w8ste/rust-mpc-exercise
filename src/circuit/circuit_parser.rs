// An enum is a good choice for a type which can be of either one of several variants.
// Since there is a fixed choice of gate types in a Bristol circuit, an enum is a natural
// way to represent it.
// A rust enum is similar to a tagged union in C/C++.

use std::usize;

use crate::circuit::circuit_error::CircuitError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateType {
    // Enum variants can have fields themselves.
    XOR(usize, usize),
    AND(usize, usize),
    INV(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gate {
    pub gate_type: GateType,
    pub output: usize,
}

// We can 'derive' some traits like Debug and Clone on types via a derive attribute. This is a
// macro which expands to the corresponding trait implementation of the trait.
// cargo-expand (https://github.com/dtolnay/cargo-expand) can show you the expanded code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub gates_amount: usize,
    pub wires_amount: usize,
    pub niv: Vec<usize>,
    pub nov: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct Circuit {
    // a circuit consists of a header and the gates of a circuit
    pub header: Header,
    pub gates: Vec<Gate>,
}

fn get_expected_line_length_header(lines: Vec<&str>, l: usize) -> Result<usize, CircuitError> {
    match lines[l].get(0..1) {
        Some(value) => match value.parse::<usize>() {
            Ok(count) => Ok(count),
            Err(_) => {
                let m = format!("Something went wrong whilst parsing line {}", l);
                Err(CircuitError::ParsingError(m))
            }
        },
        None => {
            let m = format!("Something went wrong whilst parsing line {}", l);
            Err(CircuitError::ParsingError(m))
        }
    }
}

impl Circuit {
    pub fn get_output_wires(&self) -> usize {
        self.header.wires_amount - self.get_nov_sum()
    }

    fn get_nov_sum(&self) -> usize {
        self.header.nov.iter().sum()
    }

    /// Parses the bristol file contents into a circuit
    pub fn parse(circuit: &str) -> Result<Self, CircuitError> {
        // This method parses the circuit string representation into the Circuit type
        // Split the input string into lines
        let lines: Vec<&str> = circuit.lines().collect();

        if lines.len() < 5 {
            return Err(CircuitError::ParsingError(
                "the Circuit being too small".to_string(),
            ));
        }

        // =========== Parse the header ==========
        let header_info: Vec<usize> = lines[0]
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();

        if header_info.len() != 2 {
            return Err(CircuitError::ParsingHeaderInformationError(
                2,
                header_info.len(),
            ));
        }

        // Parsing niv line

        let inputs_count = match get_expected_line_length_header(lines.clone(), 1) {
            Ok(v) => v,
            Err(e) => {
                return Err(e);
            }
        };

        let niv: Vec<usize> = lines[1]
            .split_whitespace()
            .skip(1)
            .map(|s| s.parse().unwrap())
            .collect();

        if inputs_count != niv.len() {
            return Err(CircuitError::ParsingNivError(inputs_count, niv.len()));
        }

        // Parsing Nov line

        let outputs_count = match get_expected_line_length_header(lines.clone(), 2) {
            Ok(v) => v,
            Err(e) => {
                return Err(e);
            }
        };

        let nov: Vec<usize> = lines[2]
            .split_whitespace()
            .skip(1)
            .map(|s| s.parse().unwrap())
            .collect();

        if outputs_count != nov.len() {
            return Err(CircuitError::ParsingNovError(outputs_count, nov.len()));
        }

        let header: Header = Header {
            gates_amount: header_info[0],
            wires_amount: header_info[1],
            niv,
            nov,
        };

        if !lines[3].is_empty() {
            return Err(CircuitError::EmptyLineMissingError);
        }

        // ============= parse the gates ============

        let mut gates: Vec<Gate> = Vec::new();

        for line in lines[4..].iter() {
            let gate_info: Vec<&str> = line.split_whitespace().collect();

            let input_amount: usize = gate_info[0].parse().unwrap();
            let output_amount: usize = gate_info[1].parse().unwrap();

            let gate_type: GateType = match gate_info[input_amount + output_amount + 2] {
                "XOR" => {
                    GateType::XOR(gate_info[2].parse().unwrap(), gate_info[3].parse().unwrap())
                }
                "AND" => {
                    GateType::AND(gate_info[2].parse().unwrap(), gate_info[3].parse().unwrap())
                }
                "INV" => GateType::INV(gate_info[2].parse().unwrap()),
                _ => {
                    return Err(CircuitError::NotAGateError(
                        gate_info[input_amount + output_amount + 2].to_string(),
                    ))
                }
            };

            let output_index: usize;
            if input_amount == 2 {
                output_index = 4;
            } else if input_amount == 1 {
                output_index = 3;
            } else {
                return Err(CircuitError::ParsingError(
                    "Something went wrong whilst parsing a gate".to_string(),
                ));
            }

            gates.push(Gate {
                gate_type,
                output: gate_info[output_index].parse().unwrap(),
            })
        }
        if gates.len() != header.gates_amount {
            return Err(CircuitError::WrongGateAmount(
                header.gates_amount,
                gates.len(),
            ));
        }
        Ok(Circuit { header, gates })
    }
}

// A `#[cfg(test)]` marks the following block as conditionally included only for test builds.
// cfg directives can achieve similar things as preprocessor directives in C/C++.
#[cfg(test)]
mod tests {

    use crate::circuit::circuit_parser::{Gate, GateType};

    use super::Circuit;
    // Functions marked with `#[test]` are automatically run when you execute `cargo test`.
    #[test]
    fn test_and() {
        let circuit = "\
            1 3\n\
            2 1 1\n\
            1 1\n\
            \n\
            2 1 0 1 9 AND\n";

        let c = Circuit::parse(circuit).unwrap();

        let g: Gate = Gate {
            gate_type: GateType::AND(0, 1),
            output: 9,
        };
        assert_eq!(c.gates, vec![g]);
    }

    #[test]
    fn test_xor() {
        let circuit = "\
            1 3\n\
            2 1 1\n\
            1 1\n\
            \n\
            2 1 0 1 9 XOR\n";

        let c = Circuit::parse(circuit).unwrap();

        let g: Gate = Gate {
            gate_type: GateType::XOR(0, 1),
            output: 9,
        };
        assert_eq!(c.gates, vec![g]);
    }
    #[test]
    fn test_not() {
        let circuit = "\
            1 2\n\
            1 1\n\
            1 1\n\
            \n\
            1 1 0 9 INV\n";

        let c = Circuit::parse(circuit).unwrap();

        let g: Gate = Gate {
            gate_type: GateType::INV(0),
            output: 9,
        };
        assert_eq!(c.gates, vec![g]);
    }
}
