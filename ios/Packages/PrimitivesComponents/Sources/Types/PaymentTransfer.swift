// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import Foundation
import GemstonePrimitives
import Localization
import Primitives

public struct PaymentTransfer: Sendable {
    private let asset: Asset

    public init(asset: Asset) {
        self.asset = asset
    }

    public func destination(for payment: PaymentRequest) throws -> PaymentDestination {
        if let assetId = payment.assetId, assetId != asset.id {
            throw AnyError(Localized.Errors.invalidAssetAddress(asset.name))
        }
        let address = asset.chain.checksumAddress(payment.address)
        let recipient = Recipient(name: .none, address: address, memo: payment.memo)

        guard let value = confirmableValue(of: payment, address: address) else {
            return .recipient(RecipientData(recipient: recipient, amount: payment.exactAmount))
        }
        return .confirm(
            TransferData(
                type: .transfer(asset),
                recipientData: RecipientData(recipient: recipient, amount: .none),
                amount: .exact(value),
            ),
        )
    }
}

// MARK: - Private

private extension PaymentTransfer {
    func confirmableValue(of payment: PaymentRequest, address: String) -> BigInt? {
        guard asset.chain.isValidAddress(address), !needsMemoReview(payment) else { return .none }
        return transferValue(of: payment)
    }

    func needsMemoReview(_ payment: PaymentRequest) -> Bool {
        asset.chain.isMemoSupported && payment.memo != nil
    }

    func transferValue(of payment: PaymentRequest) -> BigInt? {
        switch payment.amount {
        case let .exactValue(value): try? BigNumberFormatter.standard.exactNumber(from: value, decimals: asset.decimals.asInt)
        case let .atomicValue(value): BigInt(value)
        case .none: .none
        }
    }
}
