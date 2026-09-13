// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemVerificationLevel
import func Gemstone.verificationLevel
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct ConnectionProposalViewModel {
    private let confirmTransferDelegate: TransferDataCallback.ConfirmTransferDelegate
    private let pairingProposal: WCPairingProposal

    var walletSelectorModel: SelectWalletViewModel

    public init(
        confirmTransferDelegate: @escaping TransferDataCallback.ConfirmTransferDelegate,
        pairingProposal: WCPairingProposal,
    ) {
        self.confirmTransferDelegate = confirmTransferDelegate
        self.pairingProposal = pairingProposal
        walletSelectorModel = SelectWalletViewModel(
            wallets: pairingProposal.proposal.wallets,
            selectedWallet: pairingProposal.proposal.defaultWallet,
        )
    }

    var title: String {
        Localized.WalletConnect.Connect.title
    }

    var buttonTitle: String {
        Localized.Transfer.confirm
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
        payload.metadata.shortName
    }

    var websiteText: String? {
        let host = payload.metadata.host
        return host.isEmpty ? nil : host
    }

    var appText: String {
        AppDisplayFormatter.format(name: appName, host: websiteText)
    }

    var imageUrl: URL? {
        payload.metadata.iconURL
    }

    private var verification: GemVerificationLevel {
        verificationLevel(status: pairingProposal.verificationStatus.map())
    }

    var verificationImage: Image {
        switch verification {
        case .verified: Images.Transaction.State.success
        case .unverified: Images.TokenStatus.warning
        case .suspicious: Images.TokenStatus.risk
        }
    }

    var statusText: String {
        switch verification {
        case .verified: Localized.Asset.Verification.verified
        case .unverified: Localized.Asset.Verification.unverified
        case .suspicious: Localized.Asset.Verification.suspicious
        }
    }

    var statusTextStyle: TextStyle {
        switch verification {
        case .verified: TextStyle(font: .callout, color: Colors.green)
        case .unverified: TextStyle(font: .callout, color: Colors.orange)
        case .suspicious: TextStyle(font: .callout, color: Colors.red)
        }
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

    private var payload: WalletConnectionSessionProposal {
        pairingProposal.proposal
    }
}

// MARK: - Business Logic

extension ConnectionProposalViewModel {
    func accept() {
        guard let selectedWallet = walletSelectorModel.selectedItems.first else {
            return
        }
        confirmTransferDelegate(.success(selectedWallet.id.id))
    }
}
