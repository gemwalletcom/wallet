// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import Foundation
import GemstonePrimitives
import Localization
import Primitives

public struct PaymentTransfer: Sendable {
    private let asset: Asset
    private let numberFormatter: BigNumberFormatter

    public init(asset: Asset, numberFormatter: BigNumberFormatter = .standard) {
        self.asset = asset
        self.numberFormatter = numberFormatter
    }

    public func destination(for payment: PaymentRequest) throws -> PaymentDestination {
        guard isSameAsset(payment) else {
            throw AnyError(Localized.Errors.invalidAssetAddress(asset.name))
        }
        let address = asset.chain.checksumAddress(payment.address)
        let recipient = Recipient(name: .none, address: address, memo: payment.memo)

        switch confirmableValue(of: payment, address: address) {
        case let .some(value):
            return .confirm(
                TransferData(
                    type: .transfer(asset),
                    recipientData: RecipientData(recipient: recipient, amount: .none),
                    amount: .exact(value),
                ),
            )
        case .none:
            return .recipient(RecipientData(recipient: recipient, amount: payment.amount))
        }
    }
}

// MARK: - Private

private extension PaymentTransfer {
    func isSameAsset(_ payment: PaymentRequest) -> Bool {
        guard let assetId = payment.assetId else { return true }
        return assetId == asset.id
    }

    func confirmableValue(of payment: PaymentRequest, address: String) -> BigInt? {
        guard let value = transferValue(of: payment) else { return .none }
        guard asset.chain.isValidAddress(address), !needsMemoReview(payment) else { return .none }
        return value
    }

    func needsMemoReview(_ payment: PaymentRequest) -> Bool {
        asset.chain.isMemoSupported && payment.memo != nil
    }

    func transferValue(of payment: PaymentRequest) -> BigInt? {
        guard let amount = payment.amount else { return .none }
        return try? numberFormatter.exactNumber(from: amount, decimals: asset.decimals.asInt)
    }
}
