// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import protocol Gemstone.GemChainSettingsServiceProtocol
import enum Gemstone.GemChainSettingsSection
import struct Gemstone.GemExplorerRow
import struct Gemstone.GemNodeListSession
import struct Gemstone.GemNodeSelection
import enum Gemstone.GemNodeStatusState
import GemstonePrimitives
import Localization
import Primitives

@Observable
@MainActor
public final class ChainSettingsSceneViewModel {
    private let service: any GemChainSettingsServiceProtocol
    let chain: Chain

    var nodeDelete: GemNodeSelection?
    var explorers: [GemExplorerRow]
    var isPresentingImportNode: Bool = false
    var isPresentingAlertMessage: AlertMessage?

    private var session: GemNodeListSession

    public init(chain: Chain, service: any GemChainSettingsServiceProtocol) {
        self.chain = chain
        self.service = service
        explorers = service.explorerRows(chain: chain.rawValue)
        session = service.newNodeListSession(chain: chain.rawValue)
    }

    var title: String {
        chain.networkName
    }

    var sections: [GemChainSettingsSection] {
        service.sections()
    }

    var nodesModels: [ChainNodeViewModel] {
        service.nodeRows(chain: chain.rawValue, nodes: session.nodes, statuses: session.statuses)
            .map { ChainNodeViewModel(row: $0) }
    }

    var deleteButtonTitle: String {
        Localized.Common.delete
    }

    func deleteConfirmationTitle(for nodeName: String) -> String {
        Localized.Common.deleteConfirmation(nodeName)
    }

    func addNodeModel() -> AddNodeSceneViewModel {
        AddNodeSceneViewModel(chain: chain, service: service)
    }
}

// MARK: - Actions

extension ChainSettingsSceneViewModel {
    func load() async {
        do {
            try await loadNodes()
            await loadNodesStates()
        } catch {
            isPresentingAlertMessage = AlertMessage(error: error)
        }
    }

    func onSelectExplorer(name: String) {
        do {
            try service.setExplorerName(chain: chain.rawValue, name: name)
            explorers = service.explorerRows(chain: chain.rawValue)
        } catch {
            isPresentingAlertMessage = AlertMessage(error: error)
        }
    }

    func onSelectNode(_ url: String) {
        Task {
            do {
                try await service.selectNode(chain: chain.rawValue, url: url)
                try await loadNodes()
            } catch {
                isPresentingAlertMessage = AlertMessage(error: error)
            }
        }
    }

    func onSelectNodeForDeletion(_ node: GemNodeSelection) {
        nodeDelete = node
    }

    func onPresentImportNode() {
        isPresentingImportNode = true
    }

    func onDismissImportNode() {
        isPresentingImportNode = false
        Task {
            await load()
        }
    }

    func onDeleteNode() {
        Task {
            do {
                try await delete()
            } catch {
                isPresentingAlertMessage = AlertMessage(error: error)
            }
        }
    }
}

// MARK: - Private

extension ChainSettingsSceneViewModel {
    private func loadNodes() async throws {
        session = session.onNodes(nodes: try await service.nodes(chain: chain.rawValue))
    }

    private func loadNodesStates() async {
        session = session.onChecking()
        await withTaskGroup(of: (String, GemNodeStatusState).self) { group in
            for url in session.nodeUrls() {
                group.addTask {
                    await (url, self.service.nodeStatus(chain: self.chain.rawValue, url: url))
                }
            }

            for await (url, state) in group {
                session = session.onStatus(url: url, state: state)
            }
        }
    }

    private func delete() async throws {
        guard let nodeDelete else { return }
        try await service.deleteNode(chain: chain.rawValue, url: nodeDelete.url)
        try await loadNodes()
    }
}
