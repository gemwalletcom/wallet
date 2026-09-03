// Copyright (c). Gem Wallet. All rights reserved.

import Components
import protocol Gemstone.GemChainSettingsServiceProtocol
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

@MainActor
@Observable
final class AddNodeSceneViewModel {
    private let service: any GemChainSettingsServiceProtocol

    let chain: Chain

    var urlInputModel = InputValidationViewModel(mode: .onDemand)
    var state: StateViewType<AddNodeResultViewModel> = .noData
    var isPresentingScanner: Bool = false
    var isPresentingAlertMessage: AlertMessage?
    var loadTrigger: AddNodeLoadTrigger?

    init(chain: Chain, service: any GemChainSettingsServiceProtocol) {
        self.chain = chain
        self.service = service
    }

    var title: String {
        Localized.Nodes.ImportNode.title
    }

    var actionButtonTitle: String {
        Localized.Wallet.Import.action
    }

    var inputFieldTitle: String {
        Localized.Common.url
    }

    var errorTitle: String {
        Localized.Errors.errorOccurred
    }

    var chainModel: ChainViewModel {
        ChainViewModel(chain: chain)
    }

    var warningModel: ListItemModel {
        ListItemModel(
            title: Localized.Asset.Verification.warningTitle,
            titleStyle: .headline,
            titleExtra: Localized.Nodes.ImportNode.warningMessage,
            titleStyleExtra: .bodySecondary,
            imageStyle: ListItemImageStyle(
                assetImage: AssetImage(type: .emoji(Emoji.WalletAvatar.warning.rawValue)),
                imageSize: .image.semiMedium,
                alignment: .top,
                cornerRadiusType: .none,
            ),
        )
    }
}

// MARK: - Business Logic

extension AddNodeSceneViewModel {
    func onChangeInput() {
        guard loadTrigger?.url != urlInputModel.text else { return }
        setLoadTrigger(isImmediate: false)
    }

    func setInput(_ text: String) {
        urlInputModel.text = text
        setLoadTrigger(isImmediate: true)
    }

    private func setLoadTrigger(isImmediate: Bool) {
        let text = urlInputModel.text
        guard text.isNotEmpty else {
            state = .noData
            loadTrigger = nil
            return
        }
        loadTrigger = AddNodeLoadTrigger(url: text, isImmediate: isImmediate)
    }

    func importFoundNode() async throws {
        guard case let .data(model) = state else {
            throw AnyError("Unknown result")
        }

        try await service.addNode(chain: chain.rawValue, url: model.url)
    }

    func load() async {
        state = .loading
        do {
            state = try await .data(AddNodeResultViewModel(result: service.checkNode(chain: chain.rawValue, url: urlInputModel.text)))
        } catch {
            state.setError(error)
        }
    }
}
