use crate::{
    circuit_vals::FromCircuitVal, commitments::Commitments, utils::to_fq, Fq, MultiCommitGens,
};
use libspartan::{
    group::CompressedGroup,
    transcript::{AppendToTranscript, ProofTranscript, Transcript},
};
use secpq_curves::Secq256k1;

#[derive(Debug, Clone, Copy)]
pub struct ZKDotProdProof<const DIMENSION: usize> {
    pub delta: Secq256k1,
    pub beta: Secq256k1,
    pub z: [Fq; DIMENSION],
    pub z_delta: Fq,
    pub z_beta: Fq,
}

// Utilities
pub fn dot_prod(x: &[Fq], a: &[Fq]) -> Fq {
    let mut result = Fq::zero();
    for (x, a) in x.iter().zip(a.iter()) {
        result += *x * *a;
    }

    result
}

// https://eprint.iacr.org/2017/1132.pdf
// P.18, Figure 6, steps 4
pub fn verify<const DIMENSION: usize>(
    tau: &Secq256k1,
    a: &[Fq],
    proof: &ZKDotProdProof<DIMENSION>,
    com_poly: &Secq256k1,
    gens_1: &MultiCommitGens,
    gens_n: &MultiCommitGens,
    transcript: &mut Transcript,
) {
    transcript.append_protocol_name(b"dot product proof");

    CompressedGroup::from_circuit_val(com_poly).append_to_transcript(b"Cx", transcript);

    CompressedGroup::from_circuit_val(tau).append_to_transcript(b"Cy", transcript);

    transcript.append_message(b"a", b"begin_append_vector");
    for a_i in a {
        transcript.append_message(b"a", &a_i.to_bytes());
    }
    transcript.append_message(b"a", b"end_append_vector");

    CompressedGroup::from_circuit_val(&proof.delta).append_to_transcript(b"delta", transcript);

    CompressedGroup::from_circuit_val(&proof.beta).append_to_transcript(b"beta", transcript);

    let c = to_fq(&transcript.challenge_scalar(b"c"));

    // (13)
    let lhs = (com_poly * c) + proof.delta;
    let rhs = proof.z.commit(&proof.z_delta, gens_n);

    assert!(lhs == rhs, "dot prod verification failed (13)");

    // (14)
    let lhs = (tau * c) + proof.beta;
    let rhs = dot_prod(&proof.z, a).commit(&proof.z_beta, gens_1);

    assert!(lhs == rhs, "dot prod verification failed (14)");
}
