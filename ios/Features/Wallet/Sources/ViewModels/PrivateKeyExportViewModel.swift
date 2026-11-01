// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import typealias Gemstone.Chain
import struct Gemstone.GemPrivateKeyAccount
import enum Gemstone.GemSecretExport
import protocol Gemstone.GemWalletServiceProtocol
import Localization
import Primitives
import PrimitivesComponents
import SwiftUI

@Observable
@MainActor
final class PrivateKeyExportViewModel {
    private let walletId: WalletId
    private let export: GemSecretExport
    private let service: any GemWalletServiceProtocol

    var navigationPath = NavigationPath()
    var accounts: [GemPrivateKeyAccount] = []
    var isPresentingAlertMessage: AlertMessage?

    init(walletId: WalletId, export: GemSecretExport, service: any GemWalletServiceProtocol) {
        self.walletId = walletId
        self.export = export
        self.service = service
    }

    var title: String {
        Localized.Common.privateKey
    }

    var accountsInfo: String {
        Localized.PrivateKey.accountsInfo
    }

    var selectAccountTitle: String {
        Localized.PrivateKey.selectAccount
    }

    func listItem(for account: GemPrivateKeyAccount) -> ListItemModel {
        ListItemModel(
            title: account.row.title,
            titleExtra: account.address,
            titleStyleExtra: .calloutSecondary,
            imageStyle: .asset(assetImage: AssetImage(icon: account.row.icon)),
        )
    }
}

// MARK: - Actions

extension PrivateKeyExportViewModel {
    func onContinue() async {
        switch export {
        case let .privateKey(chain): await onSelect(chain: chain)
        case .privateKeyAccounts: navigationPath.append(PrivateKeyExportDestination.accounts)
        case .secret: break
        }
    }

    func fetchAccounts() async {
        do {
            accounts = try await service.privateKeyAccounts(walletId: walletId)
        } catch {
            isPresentingAlertMessage = AlertMessage(error: error)
        }
    }

    func onSelect(chain: Chain) async {
        do {
            let secret = try await service.exportPrivateKey(walletId: walletId, chain: chain)
            navigationPath.append(PrivateKeyExportDestination.secret(secret))
        } catch {
            isPresentingAlertMessage = AlertMessage(error: error)
        }
    }
}
