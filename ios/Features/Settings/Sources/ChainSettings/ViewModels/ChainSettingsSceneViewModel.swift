// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import protocol Gemstone.GemChainSettingsServiceProtocol
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives

@Observable
@MainActor
public final class ChainSettingsSceneViewModel {
    private let service: any GemChainSettingsServiceProtocol
    let chain: Chain

    var selectedExplorer: String?
    var selectedNode: ChainNode
    var nodeDelete: ChainNode?
    var explorers: [String]
    var isPresentingImportNode: Bool = false

    private let formatter = ValueFormatter.full_US

    private var nodes: [ChainNode] = []
    private var statusStateByNodeId: [String: NodeStatusState] = [:]

    public init(chain: Chain, service: any GemChainSettingsServiceProtocol) {
        self.chain = chain
        self.service = service
        selectedNode = chain.defaultChainNode
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
                chainNode: node,
                gemNodeFlag: service.nodeFlag(url: node.node.url),
                statusState: statusStateByNodeId[node.id] ?? .none,
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

    func canDelete(node: ChainNode) -> Bool {
        service.canDeleteNode(chain: chain.rawValue, url: node.node.url)
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
            selectedNode = try currentNode()
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

    func onSelectNode(_ node: ChainNode) {
        selectedNode = node
        Task {
            do {
                try await service.selectNode(chain: chain.rawValue, url: node.node.url)
            } catch {
                // TODO: - handle error
                debugLog("chain settings scene: on chain select error \(error)")
            }
        }
    }

    func onSelectNodeForDeletion(_ chainNode: ChainNode) {
        nodeDelete = chainNode
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
        nodes = try await service.nodes(chain: chain.rawValue).map { try ChainNode(chain: chain.rawValue, node: Primitives.Node($0)) }
    }

    private func clear() {
        statusStateByNodeId = [:]
    }

    private func loadNodesStates() async {
        await withTaskGroup(of: (ChainNode, NodeStatusState).self) { group in
            for node in nodes {
                group.addTask {
                    await (node, self.loadNodeStatusState(for: node))
                }
            }

            for await (node, state) in group {
                statusStateByNodeId[node.id] = state
            }
        }
    }

    private func delete() async throws {
        guard let nodeDelete else { return }
        try await service.deleteNode(chain: chain.rawValue, url: nodeDelete.node.url)
        selectedNode = try currentNode()
        try await loadNodes()
    }

    private func currentNode() throws -> ChainNode {
        try ChainNode(chain: chain.rawValue, node: Primitives.Node(service.selectedNode(chain: chain.rawValue)))
    }

    private func loadNodeStatusState(for node: ChainNode) async -> NodeStatusState {
        guard let url = URL(string: node.node.url) else {
            return .error(error: URLError(.badURL))
        }
        do {
            let nodeStatus = try await service.nodeStatus(chain: chain.rawValue, url: url.absoluteString).map()
            return .result(nodeStatus)
        } catch {
            return .error(error: error)
        }
    }
}
