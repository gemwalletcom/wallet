// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemExplorerServiceProtocol
import Formatters
import Foundation
import Localization
import GemstoneServices
import Primitives

@Observable
@MainActor
public final class ChainSettingsSceneViewModel {
    private let explorerService: any GemExplorerServiceProtocol

    let nodeService: NodeService
    let chainServiceFactory: ChainServiceFactory
    let chain: Chain

    var selectedExplorer: String?
    var selectedNode: ChainNode
    var nodeDelete: ChainNode?
    var explorers: [String]
    var isPresentingImportNode: Bool = false

    private let defaultNodes: [ChainNode]
    private let formatter = ValueFormatter.full_US

    private var nodes: [ChainNode] = []
    private var statusStateByNodeId: [String: NodeStatusState] = [:]

    public init(
        nodeService: NodeService,
        chainServiceFactory: ChainServiceFactory,
        explorerService: any GemExplorerServiceProtocol,
        chain: Chain,
    ) {
        self.nodeService = nodeService
        self.chainServiceFactory = chainServiceFactory
        self.explorerService = explorerService

        self.chain = chain

        defaultNodes = (try? nodeService.defaultNodes(chain: chain)) ?? []
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
        .sorted(by: { !canDelete(node: $0.chainNode) && canDelete(node: $1.chainNode) })
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
        !node.isGemNode && !defaultNodes.contains(where: { $0 == node })
    }
}

// MARK: - Actions

extension ChainSettingsSceneViewModel {
    func fetch() async {
        do {
            clear()
            try await fetchNodes()
            selectedNode = try await nodeService.getNodeSelected(chain: chain)
            await fetchNodesStates()
        } catch {
            // TODO: - handle error
            debugLog("chain settings scene: fetch error \(error)")
        }
    }

    func onSelectExplorer(name: String) {
        selectedExplorer = name
        try? explorerService.setExplorerName(chain: chain.rawValue, name: name)
    }

    func onSelectNode(_ node: ChainNode) {
        selectedNode = node
        Task {
            do {
                try await nodeService.setNodeSelected(chain: chain, node: node.node)
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
            await fetch()
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
    private func fetchNodes() async throws {
        nodes = try await nodeService.nodes(for: chain)
    }

    private func clear() {
        statusStateByNodeId = [:]
    }

    private func fetchNodesStates() async {
        await withTaskGroup(of: (ChainNode, NodeStatusState).self) { group in
            for node in nodes {
                group.addTask {
                    await (node, self.fetchNodeStatusState(for: node))
                }
            }

            for await (node, state) in group {
                statusStateByNodeId[node.id] = state
            }
        }
    }

    private func delete() async throws {
        guard let nodeDelete else { return }
        try await nodeService.delete(chain: chain, node: nodeDelete.node)
        selectedNode = try await nodeService.getNodeSelected(chain: chain)
        try await fetchNodes()
    }

    private func fetchNodeStatusState(for node: ChainNode) async -> NodeStatusState {
        guard let url = URL(string: node.node.url) else {
            return .error(error: URLError(.badURL))
        }
        let service = chainServiceFactory.service(for: chain, url: url)

        do {
            let nodeStatus = try await service.getNodeStatus(url: node.node.url)
            return .result(nodeStatus)
        } catch {
            return .error(error: error)
        }
    }
}
