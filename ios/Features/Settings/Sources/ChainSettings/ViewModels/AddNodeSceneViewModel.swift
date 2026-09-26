// Copyright (c). Gem Wallet. All rights reserved.

import Components
import func Gemstone.chainRow
import enum Gemstone.GemAddNodeError
import enum Gemstone.GemAddNodePhase
import struct Gemstone.GemAddNodeSession
import struct Gemstone.GemChainRow
import protocol Gemstone.GemChainSettingsServiceProtocol
import enum Gemstone.GemServiceError
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

    var urlInputModel = InputValidationViewModel()
    private var session: GemAddNodeSession
    var isPresentingScanner: Bool = false
    var isPresentingAlertMessage: AlertMessage?
    var loadTrigger: AddNodeLoadTrigger?
    private var loadAttempt = 0

    init(chain: Chain, service: any GemChainSettingsServiceProtocol) {
        self.chain = chain
        self.service = service
        session = service.newAddNodeSession(chain: chain.rawValue)
    }

    var state: StateViewType<[ListItemField]> {
        switch session.viewState().phase {
        case .idle: .noData
        case .checking: .loading
        case let .ready(check): .data(check.rows().map { ListItemField(title: $0.title, value: $0.text) })
        case let .failed(error): .error(AnyError(error.text))
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

    var chainModel: GemChainRow {
        chainRow(chain: chain.rawValue)
    }

    var warningModel: ListItemModel {
        ListItemModel(
            title: Localized.Asset.Verification.warningTitle,
            titleExtra: Localized.Nodes.ImportNode.warningMessage,
            titleStyleExtra: .bodySecondary,
            imageStyle: .emoji(Emoji.WalletAvatar.warning.rawValue),
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

    func onSubmitInput() {
        setLoadTrigger(isImmediate: true)
    }

    private func setLoadTrigger(isImmediate: Bool) {
        session = session.onInput(url: urlInputModel.text)
        guard session.checksUrl() else {
            loadTrigger = nil
            return
        }
        if isImmediate {
            loadAttempt += 1
        }
        loadTrigger = AddNodeLoadTrigger(url: session.url, isImmediate: isImmediate, attempt: loadAttempt)
    }

    func importFoundNode() async -> Bool {
        guard let check = session.check else { return false }
        do {
            try await service.addNode(chain: chain.rawValue, url: check.url)
            session = session.onImported()
            return true
        } catch let error as GemServiceError {
            session = session.onAddFailed(error: error)
        } catch {
            session = session.onAddFailed(error: nil)
        }
        return false
    }

    func load() async {
        session = session.onChecking()
        let url = session.url
        do {
            let check = try await service.checkNode(chain: chain.rawValue, url: url)
            session = session.onChecked(url: url, check: check)
        } catch let error as GemAddNodeError {
            session = session.onCheckFailed(url: url, error: error)
        } catch {
            session = session.onCheckFailed(url: url, error: nil)
        }
    }
}
