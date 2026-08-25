// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import GemstonePrimitives
import Localization
import Primitives

public struct PaymentTransfer: Sendable {
    public enum Destination: Sendable {
        case confirm(TransferData)
        case recipient(RecipientData)
    }

    private let asset: Asset

    public init(asset: Asset) {
        self.asset = asset
    }

    public func destination(for payment: PaymentRequest) throws -> Destination {
        let recipient = try recipient(for: payment)
        guard let value = confirmableValue(of: payment, address: recipient.address) else {
            return .recipient(RecipientData(recipient: recipient, amount: payment.exactAmount))
        }
        return .confirm(transferData(recipient: recipient, value: value))
    }

    public func decodedTransfer(for payment: PaymentRequest) -> TransferData? {
        guard let recipient = try? recipient(for: payment),
              asset.chain.isValidAddress(recipient.address),
              let value = transferValue(of: payment)
        else {
            return .none
        }
        return transferData(recipient: recipient, value: value)
    }
}

// MARK: - Private

private extension PaymentTransfer {
    func transferData(recipient: Recipient, value: BigInt) -> TransferData {
        TransferData(
            type: .transfer(asset),
            recipientData: RecipientData(recipient: recipient, amount: nil),
            amount: .exact(value),
        )
    }

    func recipient(for payment: PaymentRequest) throws -> Recipient {
        guard payment.assetId == nil || payment.assetId == asset.id else {
            throw AnyError(Localized.Errors.notSupported)
        }
        return Recipient(
            name: nil,
            address: asset.chain.checksumAddress(payment.address),
            memo: payment.memo,
            references: payment.references ?? [],
        )
    }

    func confirmableValue(of payment: PaymentRequest, address: String) -> BigInt? {
        guard asset.chain.isValidAddress(address) else { return .none }
        if asset.chain.isMemoSupported {
            guard payment.memo?.isEmpty == false else { return .none }
        }
        return transferValue(of: payment)
    }

    func transferValue(of payment: PaymentRequest) -> BigInt? {
        switch payment.amount {
        case let .exactValue(value): try? BigNumberFormatter.standard.exactNumber(from: value, decimals: asset.decimals.asInt)
        case let .atomicValue(value): BigInt(value)
        case .none: .none
        }
    }
}
