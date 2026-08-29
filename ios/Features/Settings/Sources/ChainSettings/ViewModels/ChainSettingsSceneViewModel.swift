// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import protocol Gemstone.GemExplorerServiceProtocol
import class Gemstone.GemNodeService
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives

@Observable
@MainActor
public final class ChainSettingsSceneViewModel {
    private let explorerService: any GemExplorerServiceProtocol

    let nodeService: GemNodeService
    let gatewayService: GatewayService
    let addNodeService: AddNodeService
    let chain: Chain

    var selectedExplorer: String?
    var selectedNode: ChainNode
    var nodeDelete: ChainNode?
    var explorers: [String]
    var isPresentingImportNode: Bool = false

    private let formatter = ValueFormatter.full_US

    private var nodes: [ChainNode] = []
    private var statusStateByNodeId: [String: NodeStatusState] = [:]

    public init(
        nodeService: GemNodeService,
        gatewayService: GatewayService,
        addNodeService: AddNodeService,
        explorerService: any GemExplorerServiceProtocol,
        chain: Chain,
    ) {
        self.nodeService = nodeService
        self.addNodeService = addNodeService
        self.gatewayService = gatewayService
        self.explorerService = explorerService

        self.chain = chain

        selectedNode = chain.defaultChainNode
        explorers = explorerService.getExplorers(chain: chain.rawValue)
        selectedExplorer = explorerService.getExplorerName(chain: chain.rawValue)
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
        nodeService.canDeleteNode(chain: chain.rawValue, url: node.node.url)
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
            try explorerService.setExplorerName(chain: chain.rawValue, name: name)
        } catch {
            debugLog("chain settings scene: on explorer select error \(error)")
        }
    }

    func onSelectNode(_ node: ChainNode) {
        selectedNode = node
        Task {
            do {
                try await nodeService.selectNode(chain: chain.rawValue, url: node.node.url)
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
        nodes = try await nodeService.sortedNodes(chain: chain.rawValue, nodes: nodeService.getNodes(chain: chain.rawValue))
            .map { try ChainNode(chain: chain.rawValue, node: Primitives.Node($0)) }
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
        try await nodeService.deleteNode(chain: chain.rawValue, url: nodeDelete.node.url)
        selectedNode = try currentNode()
        try await loadNodes()
    }

    private func currentNode() throws -> ChainNode {
        try ChainNode(chain: chain.rawValue, node: Primitives.Node(nodeService.selectedNode(chain: chain.rawValue)))
    }

    private func loadNodeStatusState(for node: ChainNode) async -> NodeStatusState {
        guard let url = URL(string: node.node.url) else {
            return .error(error: URLError(.badURL))
        }
        do {
            let nodeStatus = try await gatewayService.nodeStatus(chain: chain, url: url.absoluteString)
            return .result(nodeStatus)
        } catch {
            return .error(error: error)
        }
    }
}
