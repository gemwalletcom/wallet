// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemWalletService
import protocol Gemstone.GemWalletServiceProtocol
import GemstoneServices
import GemstoneServicesTestKit
import ManageWallets
import SwiftUI

public extension WalletsSceneViewModel {
    static func mock(
        walletService: any GemWalletServiceProtocol = GemWalletService.mock(),
        biometry: any BiometryAuthenticatable = BiometryAuthenticationMock(requiresAuthentication: false),
    ) -> WalletsSceneViewModel {
        WalletsSceneViewModel(
            navigationPath: .constant(NavigationPath()),
            walletService: walletService,
            biometry: biometry,
            isPresentingCreateWalletSheet: .constant(false),
            isPresentingImportWalletSheet: .constant(false),
        )
    }
}
