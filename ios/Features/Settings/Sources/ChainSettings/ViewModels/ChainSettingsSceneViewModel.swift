// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import protocol Gemstone.GemChainSettingsServiceProtocol
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

    var selectedExplorer: String?
    var nodeDelete: GemNodeSelection?
    var explorers: [String]
    var isPresentingImportNode: Bool = false

    private let formatter = ValueFormatter.full_US

    private var nodes: [GemNodeSelection] = []
    private var statusStateByNodeUrl: [String: GemNodeStatusState] = [:]

    public init(chain: Chain, service: any GemChainSettingsServiceProtocol) {
        self.chain = chain
        self.service = service
        explorers = service.explorers(chain: chain.rawValue)
        selectedExplorer = service.explorerName(chain: chain.rawValue)
    }

    var title: String {
        chain.networkName
    }

    var nodesTitle: String {
        Localized.Settings.Networks.source
    }

    var nodesModels: [ChainNodeViewModel] {
        nodes.map { node in
            ChainNodeViewModel(
                node: node,
                gemNodeFlag: service.nodeFlag(url: node.url),
                statusState: statusStateByNodeUrl[node.url] ?? .loading,
                formatter: formatter,
            )
        }
    }

    var explorerTitle: String {
        Localized.Settings.Networks.explorer
    }

    var deleteButtonTitle: String {
        Localized.Common.delete
    }

    func deleteConfirmationTitle(for nodeName: String) -> String {
        Localized.Common.deleteConfirmation(nodeName)
    }

    func canDelete(url: String) -> Bool {
        service.canDeleteNode(chain: chain.rawValue, url: url)
    }

    func addNodeModel() -> AddNodeSceneViewModel {
        AddNodeSceneViewModel(chain: chain, service: service)
    }
}

// MARK: - Actions

extension ChainSettingsSceneViewModel {
    func load() async {
        do {
            clear()
            try await loadNodes()
            await loadNodesStates()
        } catch {
            // TODO: - handle error
            debugLog("chain settings scene: load error \(error)")
        }
    }

    func onSelectExplorer(name: String) {
        selectedExplorer = name
        do {
            try service.setExplorerName(chain: chain.rawValue, name: name)
        } catch {
            debugLog("chain settings scene: on explorer select error \(error)")
        }
    }

    func onSelectNode(_ url: String) {
        Task {
            do {
                try await service.selectNode(chain: chain.rawValue, url: url)
                try await loadNodes()
            } catch {
                // TODO: - handle error
                debugLog("chain settings scene: on chain select error \(error)")
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
                // TODO: - handle error
                debugLog("chain settings scene: on delete error \(error)")
            }
        }
    }
}

// MARK: - Private

extension ChainSettingsSceneViewModel {
    private func loadNodes() async throws {
        nodes = try await service.nodes(chain: chain.rawValue)
    }

    private func clear() {
        statusStateByNodeUrl = [:]
    }

    private func loadNodesStates() async {
        await withTaskGroup(of: (String, GemNodeStatusState).self) { group in
            for url in nodes.map(\.url) {
                group.addTask {
                    await (url, self.service.nodeStatus(chain: self.chain.rawValue, url: url))
                }
            }

            for await (url, state) in group {
                statusStateByNodeUrl[url] = state
            }
        }
    }

    private func delete() async throws {
        guard let nodeDelete else { return }
        try await service.deleteNode(chain: chain.rawValue, url: nodeDelete.url)
        try await loadNodes()
    }
}
