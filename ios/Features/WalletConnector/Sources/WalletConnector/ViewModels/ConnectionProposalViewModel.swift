// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.applicationConnectionRow
import struct Gemstone.GemConnectionRow
import enum Gemstone.GemVerificationLevel
import func Gemstone.verificationLevel
import func Gemstone.walletRow
import func Gemstone.walletRows
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct ConnectionProposalViewModel {
    private let confirmTransferDelegate: TransferDataCallback.ConfirmTransferDelegate
    private let pairingProposal: WCPairingProposal
    private let row: GemConnectionRow
    private let verification: GemVerificationLevel

    var walletSelectorModel: SelectWalletViewModel

    public init(
        confirmTransferDelegate: @escaping TransferDataCallback.ConfirmTransferDelegate,
        pairingProposal: WCPairingProposal,
    ) {
        self.confirmTransferDelegate = confirmTransferDelegate
        self.pairingProposal = pairingProposal
        row = applicationConnectionRow(metadata: pairingProposal.proposal.metadata.toGem())
        verification = verificationLevel(status: pairingProposal.verificationStatus.toGem())
        walletSelectorModel = SelectWalletViewModel(
            rows: walletRows(wallets: pairingProposal.proposal.wallets.map { $0.toGem() }),
            selectedRow: walletRow(wallet: pairingProposal.proposal.defaultWallet.toGem()),
        )
    }

    var title: String {
        Localized.WalletConnect.Connect.title
    }

    var buttonTitle: String {
        Localized.Transfer.confirm
    }

    var walletListItem: ListItemModel {
        ListItemModel(title: walletTitle, subtitle: walletName)
    }

    var connectionListItem: ListItemModel {
        ListItemModel(title: connectionTitle, subtitle: connectionText)
    }

    var walletTitle: String {
        Localized.Common.wallet
    }

    var connectionTitle: String {
        Localized.WalletConnect.Connection.title
    }

    var connectionText: String {
        Localized.WalletConnect.brandName
    }

    var walletName: String {
        walletSelectorModel.selectedItems.first?.name ?? .empty
    }

    var appName: String {
        row.title
    }

    var websiteText: String? {
        row.host
    }

    var imageUrl: URL? {
        row.iconUrl.flatMap(URL.init(string:))
    }

    var verificationImage: Image {
        verification.image
    }

    var statusText: String {
        verification.title
    }

    var statusTextStyle: TextStyle {
        verification.textStyle
    }

    var statusAssetImage: AssetImage {
        .image(verificationImage)
    }

    var permissionsTitle: String {
        Localized.WalletConnect.Permissions.title
    }

    var permissions: [ListItemModel] {
        [
            ListItemModel(
                title: Localized.WalletConnect.Permissions.viewBalance,
                imageStyle: .accessory(assetImage: .image(Images.System.checkmark), fontWeight: .semibold),
            ),
            ListItemModel(
                title: Localized.WalletConnect.Permissions.approvalRequests,
                imageStyle: .accessory(assetImage: .image(Images.System.checkmark), fontWeight: .semibold),
            ),
        ]
    }

    var appPreview: AppPreviewModel {
        AppPreviewModel(
            assetImage: AssetImage(imageURL: imageUrl),
            name: appName,
            subtitleSymbol: websiteText,
        )
    }
}

// MARK: - Business Logic

extension ConnectionProposalViewModel {
    func accept() {
        guard let selectedWallet = walletSelectorModel.selectedItems.first else {
            return
        }
        confirmTransferDelegate(.success(selectedWallet.id))
    }
}
