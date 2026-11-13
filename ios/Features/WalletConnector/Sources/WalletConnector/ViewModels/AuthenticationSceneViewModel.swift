// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.Account
import func Gemstone.applicationConnectionRow
import struct Gemstone.GemConnectionRow
import enum Gemstone.GemListRow
import enum Gemstone.GemServiceError
import protocol Gemstone.GemSignMessageServiceProtocol
import enum Gemstone.GemVerificationLevel
import struct Gemstone.MessagePayloadPreview
import func Gemstone.signerFailure
import struct Gemstone.SignMessage
import func Gemstone.verificationLevel
import func Gemstone.walletRow
import func Gemstone.walletSections
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import WalletConnectorService

@Observable
@MainActor
public final class AuthenticationSceneViewModel {
    private let service: any GemSignMessageServiceProtocol
    private let request: WCAuthenticationRequest
    private let confirmTransferDelegate: TransferDataCallback.ConfirmTransferDelegate
    private let connection: GemConnectionRow
    private let verification: GemVerificationLevel
    private var approval: Result<AuthenticationApproval, any Error>

    var walletSelectorModel: SelectWalletViewModel {
        didSet { approval = Self.approval(request: request, wallet: selectedWallet, service: service) }
    }

    public var presentedSheet: SignMessageSheet?
    public var isPresentingAlertMessage: AlertMessage?
    private var isSigning = false

    public init(
        service: any GemSignMessageServiceProtocol,
        request: WCAuthenticationRequest,
        confirmTransferDelegate: @escaping TransferDataCallback.ConfirmTransferDelegate,
    ) {
        self.service = service
        self.request = request
        self.confirmTransferDelegate = confirmTransferDelegate
        connection = applicationConnectionRow(metadata: request.proposal.metadata.toGem())
        verification = verificationLevel(status: request.verificationStatus.toGem())
        walletSelectorModel = SelectWalletViewModel(
            sections: walletSections(wallets: request.proposal.wallets.map { $0.listItem.toGem() }, currentWalletId: nil),
            selectedRow: walletRow(wallet: request.proposal.defaultWallet.toGem()),
        )
        approval = Self.approval(request: request, wallet: request.proposal.defaultWallet, service: service)
        if case let .failure(error) = approval {
            isPresentingAlertMessage = AlertMessage(error: error)
        }
    }

    var title: String {
        preview?.title.text ?? Localized.WalletConnect.Connect.title
    }

    var buttonTitle: String {
        Localized.Transfer.confirm
    }

    var appPreview: AppPreviewModel {
        AppPreviewModel(
            assetImage: AssetImage(imageURL: connection.iconUrl.flatMap(URL.init(string:))),
            name: connection.title,
            subtitleSymbol: connection.host,
        )
    }

    var walletListRow: GemListRow {
        .wallet(title: .wallet, wallet: walletRow(wallet: selectedWallet.toGem()), menu: [])
    }

    var networkRow: GemListRow? {
        account.map { .network(title: .network, chain: $0.chain, name: Chain(core: $0.chain).networkName) }
    }

    var statusText: String {
        verification.title
    }

    var statusTextStyle: TextStyle {
        verification.textStyle
    }

    var statusAssetImage: AssetImage {
        .image(verification.image)
    }

    var primaryPayloadFields: [GemListRow] {
        preview?.primary ?? []
    }

    var secondaryPayloadFields: [GemListRow] {
        preview?.secondary ?? []
    }

    var hasPayloadFields: Bool {
        primaryPayloadFields.isNotEmpty || secondaryPayloadFields.isNotEmpty
    }

    var messageText: String? {
        try? approval.get().text
    }

    public var buttonState: ButtonState {
        guard case .success = approval else {
            return .disabled
        }
        return isSigning ? .loading() : .normal
    }
}

// MARK: - Actions

public extension AuthenticationSceneViewModel {
    func onSelectPayloadAddress(_ address: String) {
        guard let account else { return }
        presentedSheet = .addressDetails(ChainAddress(chain: Chain(core: account.chain), address: address))
    }

    func onViewPayloadDetails() {
        presentedSheet = .payloadDetails
    }

    func onSign(onComplete: @escaping () -> Void) {
        guard !isSigning, case let .success(approval) = approval else { return }
        isSigning = true
        Task {
            do {
                let signature = try await service.sign(walletId: approval.wallet.toGem().id, message: approval.message)
                try await request.approve(approval.wallet, signature)
                confirmTransferDelegate(.success(signature))
                onComplete()
            } catch {
                isSigning = false
                guard let error = error as? GemServiceError, case .reject = signerFailure(error: error.text()) else {
                    isPresentingAlertMessage = AlertMessage(error: error)
                    return
                }
                confirmTransferDelegate(.failure(error))
                onComplete()
            }
        }
    }
}

// MARK: - Private

private extension AuthenticationSceneViewModel {
    var selectedWallet: Wallet {
        let selectedId = walletSelectorModel.selectedItems.first?.id.id
        return request.proposal.wallets.first { $0.id.id == selectedId } ?? request.proposal.defaultWallet
    }

    var account: Gemstone.Account? {
        try? approval.get().account
    }

    var preview: MessagePayloadPreview? {
        try? approval.get().preview
    }

    static func approval(request: WCAuthenticationRequest, wallet: Wallet, service: any GemSignMessageServiceProtocol) -> Result<AuthenticationApproval, any Error> {
        Result {
            let authMessage = try request.message(wallet)
            let message = SignMessage(chain: authMessage.account.chain, signType: .siwe, data: Data(authMessage.message.utf8))
            return AuthenticationApproval(
                wallet: wallet,
                account: authMessage.account,
                message: message,
                text: authMessage.message,
                preview: service.payloadPreview(message: message),
            )
        }
    }
}

private struct AuthenticationApproval {
    let wallet: Wallet
    let account: Gemstone.Account
    let message: SignMessage
    let text: String
    let preview: MessagePayloadPreview?
}
