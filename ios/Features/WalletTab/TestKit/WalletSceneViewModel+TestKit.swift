// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServicesTestKit
import Foundation
import PreferencesTestKit
import Primitives
import PrimitivesTestKit
import WalletSessionService
import WalletSessionServiceTestKit
import WalletTab

public extension WalletSceneViewModel {
    static func mock(wallet: Wallet = .mock()) -> WalletSceneViewModel {
        WalletSceneViewModel(
            assetDiscoveryService: .mock(),
            balanceService: .mock(),
            assetsEnabler: .mock(),
            bannerService: .mock(),
            nftService: .mock(),
            observablePreferences: .mock(),
            wallet: wallet,
            isPresentingSelectedAssetInput: .constant(.none),
            isPresentingWallets: .constant(false),
        )
    }
}
