// Copyright (c). Gem Wallet. All rights reserved.

import BalanceServiceTestKit
import BannerServiceTestKit
import DiscoverAssetsServiceTestKit
import Foundation
import NFTServiceTestKit
import PreferencesTestKit
import Primitives
import PrimitivesTestKit
import WalletServiceTestKit
import WalletTab

public extension WalletSceneViewModel {
    static func mock(wallet: Wallet = .mock()) -> WalletSceneViewModel {
        WalletSceneViewModel(
            assetDiscoveryService: .mock(),
            balanceService: .mock(),
            bannerService: .mock(),
            walletService: .mock(),
            payService: .mock(),
            nftService: .mock(),
            observablePreferences: .mock(),
            wallet: wallet,
            isPresentingSelectedAssetInput: .constant(.none),
        )
    }
}
