// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemNameService
import struct Gemstone.GemPaymentRecipient
import class Gemstone.GemRecipientService
import enum Gemstone.GemRecipientType
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Transfer

public extension RecipientSceneViewModel {
    static func mock(
        asset: Asset = .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18),
        type: GemRecipientType? = nil,
        recipient: GemPaymentRecipient? = .none,
        onNavigate: TransferRouteAction = nil,
    ) -> RecipientSceneViewModel {
        RecipientSceneViewModel(
            wallet: .mock(),
            asset: asset,
            service: GemRecipientService.mock(),
            nameService: GemNameService.mock(),
            type: type ?? .asset(asset: asset.toGem()),
            recipient: recipient,
            onNavigate: onNavigate,
        )
    }
}
