// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemWalletService
import protocol Gemstone.GemWalletServiceProtocol
import GemstoneServicesTestKit
import ManageWallets
import SwiftUI

public extension WalletsSceneViewModel {
    static func mock(walletService: any GemWalletServiceProtocol = GemWalletService.mock()) -> WalletsSceneViewModel {
        WalletsSceneViewModel(
            navigationPath: .constant(NavigationPath()),
            walletService: walletService,
            preferences: .mock(),
            isPresentingCreateWalletSheet: .constant(false),
            isPresentingImportWalletSheet: .constant(false),
        )
    }
}
