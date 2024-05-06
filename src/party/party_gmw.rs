use crate::circuit::circuit_parser::{Circuit, Gate, GateType};
use crate::mul_triple::{MTProvider, MulTriple, SeededMTP};
use crate::party::errors::PartyError;
use rand::rngs::StdRng;
use rand::{thread_rng, Rng, RngCore};
use std::cell::RefCell;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::usize;

pub struct Party<T: MTProvider> {
    circuit: Circuit,
    sender: Sender<Messages>,
    receiver: Receiver<Messages>,
    pub is_p1: bool,
    mtp: RefCell<T>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Messages {
    Result(Vec<bool>),
    And { s_i: bool, s_j: bool },
    Shares { shares: Vec<bool> },
}

/// Creates a new pair of parties for the provided circuit that can communicate with each other
/// to execute the provided circuit.
pub fn new_party_pair(circuit: Circuit) -> (Party<SeededMTP<StdRng>>, Party<SeededMTP<StdRng>>) {
    let (sender0, receiver1) = channel();
    let (sender1, receiver0) = channel();

    let mut seed: [u8; 32] = Default::default();
    thread_rng().fill_bytes(&mut seed);

    let party0: Party<SeededMTP<StdRng>> = Party::new(
        circuit.clone(),
        sender0,
        receiver0,
        false,
        SeededMTP::new(seed),
    );

    let party1: Party<SeededMTP<StdRng>> =
        Party::new(circuit, sender1, receiver1, true, SeededMTP::new(seed));

    (party0, party1)
}

// Function to generate shares of inputs between parties
fn generate_shares(input: &[bool]) -> (Vec<bool>, Vec<bool>) {
    let mut rng = thread_rng();
    let public: Vec<bool> = (0..input.len()).map(|_| rng.gen::<bool>()).collect();
    let private: Vec<bool> = input
        .iter()
        .zip(public.iter())
        .map(|(&x, &m)| x ^ m)
        .collect();
    (private, public)
}

impl<T: MTProvider> Party<T> {
    /// Create a new party.
    pub fn new(
        circuit: Circuit,
        sender: Sender<Messages>,
        receiver: Receiver<Messages>,
        is_p1: bool,
        mtp: T,
    ) -> Self {
        Party {
            circuit,
            sender,
            receiver,
            is_p1,
            mtp: RefCell::new(mtp),
        }
    }

    fn evaluate_and(&self, x: bool, y: bool) -> Result<bool, PartyError> {
        let MulTriple { a, b, c } = self.mtp.borrow_mut().get_triple();

        let (s_i1, s_j1) = (x ^ a, y ^ b);

        self.sender.send(Messages::And {
            s_i: s_i1,
            s_j: s_j1,
        })?;
        let Messages::And {
            s_i: s_i2,
            s_j: s_j2,
        } = self.receiver.recv()?
        else {
            return Err(PartyError::ThreadReceivingError);
        };

        let (s_i, s_j) = (s_i1 ^ s_i2, s_j1 ^ s_j2);

        if !self.is_p1 {
            Ok(s_i & b ^ s_j & a ^ c ^ s_i & s_j)
        } else {
            Ok(s_i & b ^ s_j & a ^ c)
        }
    }

    fn get_wire_value(&self, wires: &[Option<bool>], w: usize) -> Result<bool, PartyError<'_>> {
        match wires[w] {
            Some(value) => Ok(value),
            None => {
                return Err(PartyError::WireNotSetError(w));
            }
        }
    }

    /// Executes the GMW protocol with the linked party for the stored circuit.
    pub fn execute(&mut self, input: &[bool; 64]) -> Result<Vec<bool>, PartyError> {
        // TODO change error type
        // Iterate over the stored circuit in topological order. `match` on the gate type and
        // evaluate it, potentially using a multiplication triple for and And Gate and communication
        // over the shared channel.

        let circuit = &self.circuit;

        let mut wires: Vec<Option<bool>> = vec![None; circuit.header.wires_amount];

        let (mut private_share, public_share): (Vec<bool>, Vec<bool>) = generate_shares(input);

        self.sender.send(Messages::Shares {
            shares: public_share,
        })?;

        let Messages::Shares {
            shares: mut others_shares,
        } = self.receiver.recv()?
        else {
            return Err(PartyError::ThreadReceivingError);
        };

        let share = if self.is_p1 {
            private_share.extend_from_slice(&others_shares);
            private_share
        } else {
            others_shares.extend_from_slice(&private_share);
            others_shares
        };

        for (i, &wire) in share.iter().enumerate() {
            wires[i] = Some(wire);
        }

        for Gate { gate_type, output } in &circuit.gates {
            let output_index: usize = *output;
            match *gate_type {
                GateType::INV(a) => {
                    let input = match self.get_wire_value(&wires, a) {
                        Ok(v) => v,
                        Err(e) => {
                            return Err(e);
                        }
                    };
                    if self.is_p1 {
                        wires[output_index] = Some(!input);
                    } else {
                        wires[output_index] = Some(input);
                    }
                }
                GateType::XOR(a, b) => {
                    let input1 = match self.get_wire_value(&wires, a) {
                        Ok(v) => v,
                        Err(e) => {
                            return Err(e);
                        }
                    };

                    let input2 = match self.get_wire_value(&wires, b) {
                        Ok(v) => v,
                        Err(e) => {
                            return Err(e);
                        }
                    };

                    wires[output_index] = Some(input1 ^ input2);
                }
                GateType::AND(a, b) => {
                    let input1 = match self.get_wire_value(&wires, a) {
                        Ok(v) => v,
                        Err(e) => {
                            return Err(e);
                        }
                    };

                    let input2 = match self.get_wire_value(&wires, b) {
                        Ok(v) => v,
                        Err(e) => {
                            return Err(e);
                        }
                    };

                    wires[output_index] = Some(self.evaluate_and(input1, input2)?);
                }
            }
        }

        let output_offset = circuit.get_output_wires();
        let sol1: Vec<bool> = wires
            .into_iter()
            .skip(output_offset)
            .map(Option::unwrap)
            .collect();

        self.sender.send(Messages::Result(sol1.clone()))?;
        let Messages::Result(sol2) = self.receiver.recv()? else {
            return Err(PartyError::ThreadReceivingError);
        };

        Ok(sol1.iter().zip(sol2.iter()).map(|(x, y)| x ^ y).collect())
    }
}
