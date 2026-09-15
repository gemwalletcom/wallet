// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemPaymentLoad
import enum Gemstone.PaymentLink
import Foundation
import struct Gemstone.GemTransferData
import GemstonePrimitives
import Primitives
import PrimitivesComponents

public enum PaymentDestination: Identifiable, Sendable {
    case confirm(GemTransferData)
    case verify(URL, link: PaymentLink)
    case recipient(SelectedAssetInput)
    case selectAsset(SelectAssetType, chains: [Chain])

    public var id: String {
        switch self {
        case let .confirm(transfer): "confirm-\(transfer.id)"
        case let .verify(url, _): "verify-\(url)"
        case let .recipient(input): "recipient-\(input.id)"
        case let .selectAsset(type, _): "selectAsset-\(type.id)"
        }
    }

    public init(_ load: GemPaymentLoad) throws {
        switch load {
        case let .sign(transfer):
            self = .confirm(transfer)
        case let .verify(invoice, _, url):
            guard let url = URL(string: url) else {
                throw AnyError("Invalid payment verification URL")
            }
            self = .verify(url, link: invoice.link)
        }
    }
}
