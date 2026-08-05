// Copyright (c). Gem Wallet. All rights reserved.

import AvatarService
import Foundation
import enum Keystore.KeystoreImportType
import Primitives
import PrimitivesComponents
import SwiftUI
import WalletService
import WalletSessionService

@Observable
@MainActor
public final class ImportWalletViewModel {
    let walletService: WalletService
    let walletSessionService: any WalletSessionManageable
    let avatarService: AvatarService
    let nameService: any NameServiceable
    let onComplete: VoidAction

    var isPresentingSelectImageWallet: Wallet?

    public init(
        walletService: WalletService,
        walletSessionService: any WalletSessionManageable,
        avatarService: AvatarService,
        nameService: any NameServiceable,
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
