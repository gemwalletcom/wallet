// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemWalletService
import protocol Gemstone.GemWalletServiceProtocol
import GemstoneServicesTestKit
import ManageWallets
import Primitives
import PrimitivesTestKit
import SwiftUI

public extension WalletDetailViewModel {
    static func mock(
        wallet: Wallet = .mock(),
        service: any GemWalletServiceProtocol = GemWalletService.mock(),
    ) -> WalletDetailViewModel {
        WalletDetailViewModel(
            navigationPath: .constant(NavigationPath()),
            wallet: wallet,
            service: service,
            preferences: .mock(),
        )
    }
}
