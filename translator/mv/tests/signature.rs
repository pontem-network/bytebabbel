use move_binary_format::file_format::{Signature, SignatureToken};
use mv::function::signature::SignatureWriter;
use test_infra::log_init;

#[test]
pub fn test_signature() {
    log_init();

    let mut sign = SignatureWriter::default();
    sign.make_signature(vec![SignatureToken::U64]);
    sign.make_signature(vec![SignatureToken::U128]);
    sign.make_signature(vec![SignatureToken::U128]);
    sign.make_signature(vec![SignatureToken::U64]);
    sign.make_signature(vec![]);
    sign.make_signature(vec![
        SignatureToken::Address,
        SignatureToken::U8,
        SignatureToken::U128,
        SignatureToken::Signer,
    ]);
    sign.make_signature(vec![SignatureToken::U64]);
    sign.make_signature(vec![SignatureToken::U64]);
    sign.make_signature(vec![
        SignatureToken::Address,
        SignatureToken::U8,
        SignatureToken::U128,
        SignatureToken::Signer,
    ]);
    sign.make_signature(vec![
        SignatureToken::U64,
        SignatureToken::U128,
        SignatureToken::Signer,
    ]);
    sign.make_signature(vec![SignatureToken::Vector(Box::new(
        SignatureToken::Address,
    ))]);

    assert_eq!(
        sign.freeze(),
        vec![
            Signature(vec![SignatureToken::U64]),
            Signature(vec![SignatureToken::U128]),
            Signature(vec![]),
            Signature(vec![
                SignatureToken::Address,
                SignatureToken::U8,
                SignatureToken::U128,
                SignatureToken::Signer,
            ]),
            Signature(vec![
                SignatureToken::U64,
                SignatureToken::U128,
                SignatureToken::Signer,
            ]),
            Signature(vec![SignatureToken::Vector(Box::new(
                SignatureToken::Address,
            ))])
        ]
    );
}
