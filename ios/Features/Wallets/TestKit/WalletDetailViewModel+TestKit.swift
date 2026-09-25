// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemWalletService
import protocol Gemstone.GemWalletServiceProtocol
import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import SwiftUI
import Wallets

public extension WalletDetailViewModel {
    static func mock(
        wallet: Wallet = .mock(),
        service: any GemWalletServiceProtocol = GemWalletService.mock(),
        biometry: any BiometryAuthenticatable = BiometryAuthenticationMock(requiresAuthentication: false),
    ) -> WalletDetailViewModel {
        WalletDetailViewModel(
            navigationPath: .constant(NavigationPath()),
            wallet: wallet,
            service: service,
            biometry: biometry,
        )
    }
}
