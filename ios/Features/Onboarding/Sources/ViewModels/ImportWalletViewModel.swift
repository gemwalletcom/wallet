// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemAvatarServiceProtocol
import protocol Gemstone.GemChainServiceProtocol
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
    let avatarService: any GemAvatarServiceProtocol
    let nameService: any GemNameServiceProtocol
    let chainService: any GemChainServiceProtocol
    let onComplete: VoidAction

    var isPresentingSelectImageWallet: Wallet?

    public init(
        walletService: WalletService,
        walletSessionService: any WalletSessionManageable,
        avatarService: any GemAvatarServiceProtocol,
        nameService: any GemNameServiceProtocol,
        chainService: any GemChainServiceProtocol,
        onComplete: VoidAction,
    ) {
        self.walletService = walletService
        self.walletSessionService = walletSessionService
        self.avatarService = avatarService
        self.nameService = nameService
        self.chainService = chainService
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
