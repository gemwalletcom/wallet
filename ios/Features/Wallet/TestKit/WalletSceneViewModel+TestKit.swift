// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Store
import Wallet

public extension WalletSceneViewModel {
    static func mock(
        wallet: Wallet = .mock(),
        db: DB? = nil,
        service: GemWalletHomeServiceMock = GemWalletHomeServiceMock(),
    ) -> WalletSceneViewModel {
        let model = WalletSceneViewModel(
            service: service,
            observablePreferences: .mock(),
            wallet: wallet,
            isPresentingSelectedAssetInput: .constant(.none),
            isPresentingWallets: .constant(false),
            isPresentingSheet: .constant(.none),
        )
        if let db {
            model.walletQuery.bind(dbPool: db.dbPool)
            model.assetsQuery.bind(dbPool: db.dbPool)
            model.bannersQuery.bind(dbPool: db.dbPool)
        }
        return model
    }
}
