// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import WalletConnectSigner

struct DefaultCryptoProvider: CryptoProvider {
    func recoverPubKey(signature _: EthereumSignature, message _: Data) throws -> Data {
//        let publicKey = try EthereumPublicKey(
//            message: message.bytes,
//            v: EthereumQuantity(quantity: BigUInt(signature.v)),
//            r: EthereumQuantity(signature.r),
//            s: EthereumQuantity(signature.s)
//        )
//        return Data(publicKey.rawPublicKey)
        Data()
    }

    func keccak256(_: Data) -> Data {
//        let digest = SHA3(variant: .keccak256)
//        let hash = digest.calculate(for: [UInt8](data))
//        return Data(hash)
//
        Data()
    }
}
