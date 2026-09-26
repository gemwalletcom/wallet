// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemChainSettingsSection
import protocol Gemstone.GemChainSettingsServiceProtocol
import struct Gemstone.GemExplorerRow
import struct Gemstone.GemNodeListSession
import struct Gemstone.GemNodeRow
import enum Gemstone.GemNodeStatusState
import GemstonePrimitives
import Localization
import Primitives

@Observable
@MainActor
public final class ChainSettingsSceneViewModel {
    private let service: any GemChainSettingsServiceProtocol
    let chain: Chain

    var nodeDelete: GemNodeRow?
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
        session.sections(explorers: explorers)
    }

    var deleteButtonTitle: String {
        Localized.Common.delete
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

    func onSelectNodeForDeletion(_ row: GemNodeRow) {
        nodeDelete = row
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

    func onDeleteNode() async {
        do {
            try await delete()
        } catch {
            isPresentingAlertMessage = AlertMessage(error: error)
        }
    }
}

// MARK: - Private

extension ChainSettingsSceneViewModel {
    private func loadNodes() async throws {
        session = try await session.onNodes(nodes: service.nodes(chain: chain.rawValue))
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
        try await service.deleteNode(chain: chain.rawValue, url: nodeDelete.node.url)
        try await loadNodes()
    }
}
