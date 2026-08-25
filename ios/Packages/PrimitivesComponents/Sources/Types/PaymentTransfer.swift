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
        if let data = try confirmation(for: payment, type: .transfer(asset)) {
            return .confirm(data)
        }
        return .recipient(
            RecipientData(
                recipient: try recipient(for: payment),
                amount: payment.exactAmount,
            ),
        )
    }
}

extension PaymentTransfer {
    func confirmation(for payment: PaymentRequest, type: TransferDataType) throws -> TransferData? {
        let recipient = try recipient(for: payment)
        guard let value = confirmableValue(of: payment, address: recipient.address) else {
            return nil
        }
        return TransferData(
            type: type,
            recipientData: RecipientData(recipient: recipient, amount: nil),
            amount: .exact(value),
        )
    }
}

// MARK: - Private

private extension PaymentTransfer {
    func recipient(for payment: PaymentRequest) throws -> Recipient {
        guard payment.assetId == nil || payment.assetId == asset.id else {
            throw AnyError(Localized.Errors.notSupported)
        }
        return Recipient(
            name: nil,
            address: asset.chain.checksumAddress(payment.address),
            memo: payment.memo,
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
