// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemAddAssetPhase
import protocol Gemstone.GemAddAssetServiceProtocol
import struct Gemstone.GemAddAssetSession
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

    private var session: GemAddAssetSession
    var input: AddAssetInput

    var isPresentingScanner = false
    var loadTrigger: AddAssetLoadTrigger?
    var isPresentingAlertMessage: AlertMessage?

    public init(wallet: Wallet, service: any GemAddAssetServiceProtocol) {
        self.service = service
        self.wallet = wallet
        let chains = service.chains(wallet: wallet)
        let input = AddAssetInput(chains: chains, chain: service.defaultChain(chains: chains))
        session = service.newSession(chain: input.chain?.rawValue)
        self.input = input
    }

    var sections: [GemListSection] {
        service.sections(session: session)
    }

    var isLoading: Bool {
        session.isLoading
    }

    var showsVerificationWarning: Bool {
        session.viewState().canAdd
    }

    var buttonState: ButtonState {
        switch session.viewState().phase {
        case .loading: .loading()
        case .found: .normal
        case .idle, .failed: .disabled
        }
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
            imageStyle: warningImageStyle,
            infoAction: infoAction,
        )
    }

    var warningImageStyle: ListItemImageStyle? {
        ListItemImageStyle(
            assetImage: AssetImage(type: .emoji(Emoji.WalletAvatar.warning.rawValue)),
            imageSize: .image.semiMedium,
            alignment: .top,
            cornerRadiusType: .none,
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
        Task {
            do {
                try await service.add(wallet: wallet.toGem(), assetId: asset.id.identifier)
                onComplete?()
            } catch {
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
        loadTrigger = AddAssetLoadTrigger(chain: chain, address: address, isImmediate: isImmediate)
    }
}
