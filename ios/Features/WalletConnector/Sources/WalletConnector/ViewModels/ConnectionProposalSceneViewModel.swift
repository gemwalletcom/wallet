// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.applicationConnectionRow
import func Gemstone.connectionProposal
import struct Gemstone.GemConnectionProposal
import struct Gemstone.GemConnectionRow
import func Gemstone.walletRow
import func Gemstone.walletSections
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct ConnectionProposalSceneViewModel {
    private let confirmTransferDelegate: TransferDataCallback.ConfirmTransferDelegate
    private let pairingProposal: WCPairingProposal
    private let row: GemConnectionRow

    var walletSelectorModel: SelectWalletViewModel

    public init(
        confirmTransferDelegate: @escaping TransferDataCallback.ConfirmTransferDelegate,
        pairingProposal: WCPairingProposal,
    ) {
        self.confirmTransferDelegate = confirmTransferDelegate
        self.pairingProposal = pairingProposal
        row = applicationConnectionRow(metadata: pairingProposal.proposal.metadata.toGem())
        walletSelectorModel = SelectWalletViewModel(
            sections: walletSections(wallets: pairingProposal.proposal.wallets.map { $0.toGem() }, currentWalletId: nil),
            selectedRow: walletRow(wallet: pairingProposal.proposal.defaultWallet.toGem()),
        )
    }

    var title: String {
        Localized.WalletConnect.Connect.title
    }

    var buttonTitle: String {
        Localized.Transfer.confirm
    }

    var proposal: GemConnectionProposal {
        connectionProposal(status: pairingProposal.verificationStatus.toGem(), walletName: walletSelectorModel.selectedItems.first?.name ?? .empty)
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
        proposal.verification.image
    }

    var statusText: String {
        proposal.verification.title
    }

    var statusTextStyle: TextStyle {
        proposal.verification.textStyle
    }

    var statusAssetImage: AssetImage {
        .image(verificationImage)
    }

    var permissionsTitle: String {
        Localized.WalletConnect.Permissions.title
    }

    var permissions: [ListItemModel] {
        proposal.permissions.map {
            ListItemModel(
                title: $0.title,
                imageStyle: .accessory(assetImage: .image(Images.System.checkmark), fontWeight: .semibold),
            )
        }
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

extension ConnectionProposalSceneViewModel {
    func accept() {
        guard let selectedWallet = walletSelectorModel.selectedItems.first else {
            return
        }
        confirmTransferDelegate(.success(selectedWallet.id))
    }
}
