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
        asset: Asset = .mockEthereum(),
        type: GemRecipientType = .asset(asset: Asset.mock().toGem()),
        recipient: GemPaymentRecipient? = .none,
        onRecipientDataAction: RecipientDataAction = nil,
        onTransferAction: TransferDataAction = nil,
    ) -> RecipientSceneViewModel {
        RecipientSceneViewModel(
            wallet: .mock(),
            asset: asset,
            service: GemRecipientService.mock(),
            nameService: GemNameService.mock(),
            type: type,
            recipient: recipient,
            onRecipientDataAction: onRecipientDataAction,
            onTransferAction: onTransferAction,
        )
    }
}
