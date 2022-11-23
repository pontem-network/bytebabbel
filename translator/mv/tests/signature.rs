use move_binary_format::file_format::{Signature, SignatureToken};

use mv::translator::signature::SignatureWriter;

#[test]
pub fn test_signature() {
    let mut sign = SignatureWriter::new(vec![]);
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
            ))]),
        ]
    );
}
