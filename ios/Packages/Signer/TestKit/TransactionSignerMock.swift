// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Signer

public struct TransactionSignerMock: TransactionSigneable {
    public let signedData: [String]

    public init(signedData: [String] = ["signed_data"]) {
        self.signedData = signedData
    }

    public func sign(
        transfer _: TransferData,
        transactionData _: TransactionData,
        amount _: TransferAmount,
        wallet _: Wallet,
    ) throws -> [String] {
        signedData
    }
}
