// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import protocol Gemstone.GemWalletServiceProtocol
import GemstonePrimitives

@MainActor
@Observable
public final class SetupWalletViewModel: Sendable {
    private let service: any GemWalletServiceProtocol
    private let onSelectImageAction: (Wallet) -> Void
    private let onCompleteAction: (Wallet) -> Void

    public let query: ObservableQuery<WalletRequest>
    var wallet: Wallet {
        query.value
    }

    var nameInput: String

    public init(
        wallet: Wallet,
        service: any GemWalletServiceProtocol,
        onSelectImage: @escaping (Wallet) -> Void,
        onComplete: @escaping (Wallet) -> Void,
    ) {
        self.service = service
        nameInput = wallet.name
        query = ObservableQuery(WalletRequest(walletId: wallet.id), initialValue: wallet)
        onSelectImageAction = onSelectImage
        onCompleteAction = onComplete
    }

    var title: String {
        switch wallet.source {
        case .create: Localized.Wallet.New.title
        case .import: Localized.Wallet.Import.title
        }
    }

    var avatarAssetImage: AssetImage {
        let avatar = WalletViewModel(wallet: wallet).avatarImage
        return AssetImage(
            type: avatar.type,
            imageURL: avatar.imageURL,
            placeholder: avatar.placeholder,
            chainPlaceholder: Images.Wallets.editFilled,
        )
    }

    func onSelectImage() {
        onSelectImageAction(wallet)
    }

    func onComplete() {
        onCompleteAction(wallet)
    }

    func onChangeWalletName() async {
        do {
            try await service.rename(walletId: wallet.id, newName: nameInput)
        } catch {
            debugLog("Rename wallet error: \(error)")
        }
    }
}
