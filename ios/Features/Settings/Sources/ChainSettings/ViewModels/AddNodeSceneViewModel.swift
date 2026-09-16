// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemAddNodeError
import enum Gemstone.GemAddNodeFailure
import enum Gemstone.GemAddNodePhase
import struct Gemstone.GemAddNodeSession
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
    private var session: GemAddNodeSession
    var isPresentingScanner: Bool = false
    var isPresentingAlertMessage: AlertMessage?
    var loadTrigger: AddNodeLoadTrigger?

    var nodeCheckDebounce: Duration {
        .milliseconds(service.nodeCheckDebounceMilliseconds())
    }

    init(chain: Chain, service: any GemChainSettingsServiceProtocol) {
        self.chain = chain
        self.service = service
        session = service.newAddNodeSession(chain: chain.rawValue)
    }

    var state: StateViewType<AddNodeResultViewModel> {
        switch session.viewState().phase {
        case .idle: .noData
        case .checking: .loading
        case let .ready(check): .data(AddNodeResultViewModel(result: check))
        case let .failed(failure): .error(failure.error)
        }
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
        session = session.onInput(url: urlInputModel.text)
        guard session.checksUrl() else {
            loadTrigger = nil
            return
        }
        loadTrigger = AddNodeLoadTrigger(url: session.url, isImmediate: isImmediate)
    }

    func importFoundNode() async throws {
        guard let check = session.check else {
            throw AnyError("Unknown result")
        }
        try await service.addNode(chain: chain.rawValue, url: check.url)
        session = session.onImported()
    }

    func load() async {
        session = session.onChecking()
        do {
            session = try await session.onChecked(check: service.checkNode(chain: chain.rawValue, url: session.url))
        } catch let error as GemAddNodeError {
            session = session.onFailed(failure: error.failure)
        } catch {
            session = session.onFailed(failure: .unavailable)
        }
    }
}
