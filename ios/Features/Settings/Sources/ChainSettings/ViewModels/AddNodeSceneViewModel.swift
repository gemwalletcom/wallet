// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemNodeStatusServiceProtocol
import Components
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI
import Validators
import class Gemstone.GemNodeService

@MainActor
@Observable
final class AddNodeSceneViewModel {
    private let nodeService: GemNodeService
    private let nodeStatusService: any GemNodeStatusServiceProtocol

    let chain: Chain

    var urlInputModel = InputValidationViewModel(mode: .onDemand, validators: [.url])
    var state: StateViewType<AddNodeResultViewModel> = .noData
    var isPresentingScanner: Bool = false
    var isPresentingAlertMessage: AlertMessage?
    var loadTrigger: AddNodeLoadTrigger?

    init(chain: Chain, nodeService: GemNodeService, nodeStatusService: any GemNodeStatusServiceProtocol) {
        self.chain = chain
        self.nodeService = nodeService
        self.nodeStatusService = nodeStatusService
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
        guard text.isNotEmpty, urlInputModel.isValid else {
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

        // TODO: - implement disable after user selects "import node button", we can't use state: StateViewType<ImportNodeResult> progress
        try await nodeService.addNode(chain: chain.rawValue, url: model.url.absoluteString)

        // TODO: - implement correct way of selection node
        /*
         try nodeService.setNodeSelected(chain: chain, node: node)
          */
    }

    func load() async {
        guard let url = try? URLDecoder().decode(urlInputModel.text) else {
            // safety check for onSubmitUrl
            state = .error(AnyError(AddNodeError.invalidURL.errorDescription ?? ""))
            return
        }

        state = .loading

        do {
            let status = try await nodeStatusService.checkNode(chain: chain.rawValue, url: url.absoluteString).map()
            state = .data(AddNodeResultViewModel(addNodeResult: AddNodeResult(
                url: url,
                chainID: status.chainId,
                blockNumber: status.latestBlockNumber,
                isInSync: true,
                latency: status.latency,
            )))
        } catch {
            state.setError(error)
        }
    }
}
