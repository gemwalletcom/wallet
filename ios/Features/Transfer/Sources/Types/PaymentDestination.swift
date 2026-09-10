// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemPaymentLoad
import struct Gemstone.GemTransferData
import GemstonePrimitives
import Primitives
import PrimitivesComponents

public enum PaymentDestination: Identifiable, Sendable {
    case confirm(GemTransferData)
    case verify(PaymentVerification)
    case recipient(SelectedAssetInput)
    case selectAsset(SelectAssetType, chains: [Chain])

    public var id: String {
        switch self {
        case let .confirm(transfer): "confirm-\(transfer.id)"
        case let .verify(verification): "verify-\(verification.id)"
        case let .recipient(input): "recipient-\(input.id)"
        case let .selectAsset(type, _): "selectAsset-\(type.id)"
        }
    }

    public init(_ load: GemPaymentLoad) throws {
        switch load {
        case let .sign(transfer):
            self = .confirm(transfer)
        case let .verify(invoice, assetId, url):
            self = try .verify(PaymentVerification(url: url, invoice: invoice, assetId: AssetId(core: assetId)))
        }
    }
}
