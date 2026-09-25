// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import NFT
import NFTTestKit
import Primitives
import PrimitivesTestKit
import Store
import WalletTab

public extension WalletSceneViewModel {
    static func mock(
        wallet: Wallet = .mock(),
        db: DB? = nil,
        service: GemWalletHomeServiceMock = GemWalletHomeServiceMock(),
    ) -> WalletSceneViewModel {
        let model = WalletSceneViewModel(
            service: service,
            observablePreferences: .mock(),
            collectionsModel: .mock(wallet: wallet),
            wallet: wallet,
            isPresentingSelectedAssetInput: .constant(.none),
            isPresentingWallets: .constant(false),
        )
        if let db {
            model.walletQuery.bind(dbQueue: db.dbQueue)
            model.assetsQuery.bind(dbQueue: db.dbQueue)
            model.bannersQuery.bind(dbQueue: db.dbQueue)
        }
        return model
    }
}
