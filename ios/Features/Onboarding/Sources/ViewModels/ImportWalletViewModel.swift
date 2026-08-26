// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemNameServiceProtocol
import GemstoneServices
import Foundation
import Primitives
import PrimitivesComponents
import SwiftUI

@Observable
@MainActor
public final class ImportWalletViewModel {
    let walletService: WalletService
    let walletSessionService: any WalletSessionManageable
    let avatarService: AvatarService
    let nameService: any GemNameServiceProtocol
    let onComplete: VoidAction

    var isPresentingSelectImageWallet: Wallet?

    public init(
        walletService: WalletService,
        walletSessionService: any WalletSessionManageable,
        avatarService: AvatarService,
        nameService: any GemNameServiceProtocol,
        onComplete: VoidAction,
    ) {
        self.walletService = walletService
        self.walletSessionService = walletSessionService
        self.avatarService = avatarService
        self.nameService = nameService
        self.onComplete = onComplete
    }

    public var isAcceptTermsCompleted: Bool {
        walletService.isAcceptTermsCompleted
    }
}

// MARK: - Actions

extension ImportWalletViewModel {
    func presentSelectImage(wallet: Wallet) {
        isPresentingSelectImageWallet = wallet
    }
}
