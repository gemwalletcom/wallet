// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemAddAssetPhase
import protocol Gemstone.GemAddAssetServiceProtocol
import struct Gemstone.GemAddAssetSession
import struct Gemstone.GemAddAssetViewState
import struct Gemstone.GemListSection
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

@Observable
@MainActor
public final class AddAssetSceneViewModel {
    private let service: any GemAddAssetServiceProtocol
    private let wallet: Wallet

    private var session: GemAddAssetSession {
        didSet {
            viewState = session.viewState()
            sections = service.sections(session: session)
        }
    }

    private var viewState: GemAddAssetViewState
    private(set) var sections: [GemListSection]
    var input: AddAssetInput

    var isPresentingScanner = false
    var loadTrigger: AddAssetLoadTrigger?
    private var loadAttempt = 0
    var isPresentingAlertMessage: AlertMessage?

    public init(wallet: Wallet, service: any GemAddAssetServiceProtocol) {
        self.service = service
        self.wallet = wallet
        let picker = service.chainPicker(wallet: wallet.toGem())
        let input = AddAssetInput(
            chains: picker.chains.map { Chain(core: $0) },
            chain: picker.defaultChain.map { Chain(core: $0) },
            showsChainPicker: picker.showsPicker,
        )
        let session = service.newSession(chain: input.chain?.rawValue)
        self.session = session
        viewState = session.viewState()
        sections = service.sections(session: session)
        self.input = input
    }

    var isLoading: Bool {
        session.isLoading
    }

    var showsVerificationWarning: Bool {
        viewState.canAdd
    }

    var buttonState: ButtonState {
        viewState.button.state
    }

    var title: String {
        Localized.Wallet.AddToken.title
    }

    var networksModel: NetworkSelectorViewModel {
        NetworkSelectorViewModel(state: .data(.plain(input.chains)))
    }

    var networkTitle: String {
        Localized.Transfer.network
    }

    var actionButtonTitle: String {
        Localized.Wallet.Import.action
    }

    var addressTitleField: String {
        Localized.Wallet.Import.contractAddressField
    }

    var pasteImage: Image {
        Images.System.paste
    }

    var qrImage: Image {
        Images.System.qrCodeViewfinder
    }

    var addressBinding: Binding<String> {
        Binding(
            get: { [self] in
                input.address ?? ""
            },
            set: { [self] in
                input.address = $0.isEmpty ? nil : $0
            },
        )
    }

    func warningListItem(infoAction: @escaping () -> Void) -> ListItemModel {
        ListItemModel(
            title: Localized.Asset.Verification.warningTitle,
            titleExtra: Localized.Asset.Verification.warningMessage,
            titleStyleExtra: .bodySecondary,
            imageStyle: .emoji(Emoji.WalletAvatar.warning.rawValue),
            infoAction: infoAction,
        )
    }

    var tokenVerificationUrl: URL {
        AppUrl.docs(.tokenVerification)
    }

    var customTokenUrl: URL {
        AppUrl.docs(.addCustomToken)
    }
}

// MARK: - Business Logic

extension AddAssetSceneViewModel {
    func setInput(_ address: String) {
        input.address = address
        setLoadTrigger(isImmediate: true)
    }

    func onChangeAddress() {
        guard loadTrigger?.address != input.address else { return }
        setLoadTrigger(isImmediate: false)
    }

    func onSubmitAddress() {
        setLoadTrigger(isImmediate: true)
    }

    func load() async {
        guard let trigger = loadTrigger else { return }
        session = session.onLoading()
        do {
            let asset = try await service.token(chain: trigger.chain.rawValue, address: trigger.address)
            session = session.onFound(chain: trigger.chain.rawValue, address: trigger.address, asset: asset)
        } catch {
            session = session.onFailed(chain: trigger.chain.rawValue, address: trigger.address)
        }
    }

    func onSelectImportToken(onComplete: VoidAction) {
        guard let asset = session.asset?.toPrimitives() else { return }
        session = session.onAdding(isAdding: true)
        Task {
            do {
                try await service.add(wallet: wallet.toGem(), assetId: asset.id.identifier)
                onComplete?()
            } catch {
                session = session.onAdding(isAdding: false)
                isPresentingAlertMessage = AlertMessage(error: error)
            }
        }
    }

    private func setLoadTrigger(isImmediate: Bool) {
        session = session.onChain(chain: input.chain?.rawValue).onAddress(address: input.address ?? "")
        guard session.searchesToken(), let chain = input.chain, let address = input.address else {
            loadTrigger = nil
            return
        }
        if isImmediate {
            loadAttempt += 1
        }
        loadTrigger = AddAssetLoadTrigger(chain: chain, address: address, isImmediate: isImmediate, attempt: loadAttempt)
    }
}
