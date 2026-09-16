// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone

final class GemChainSettingsServiceMock: GemChainSettingsServiceProtocol, @unchecked Sendable {
    var nodesByCall: [[GemNodeSelection]] = []
    var statusByUrl: [String: GemNodeStatusState] = [:]
    var explorerRowsValue: [GemExplorerRow] = []
    var checkResult: Result<GemNodeCheck, GemAddNodeError> = .failure(.InvalidUrl)

    private(set) var selectedNodes: [String] = []
    private(set) var deletedNodes: [String] = []
    private(set) var addedNodes: [String] = []
    private(set) var setExplorerNames: [String] = []
    private(set) var nodesCalls = 0
    private(set) var statusCalls: [String] = []

    func addNode(chain _: Chain, url: String) async throws {
        addedNodes.append(url)
    }

    func chains(query _: String) -> [Chain] { [] }

    func checkNode(chain _: Chain, url _: String) async throws -> GemNodeCheck {
        try checkResult.get()
    }

    func deleteNode(chain _: Chain, url: String) async throws {
        deletedNodes.append(url)
    }

    func explorerRows(chain _: Chain) -> [GemExplorerRow] { explorerRowsValue }

    func newAddNodeSession(chain: Chain) -> GemAddNodeSession {
        GemAddNodeSession(chain: chain, url: "", check: nil, failure: nil, isChecking: false)
    }

    func newNodeListSession(chain: Chain) -> GemNodeListSession {
        GemNodeListSession(chain: chain, nodes: [], statuses: [:])
    }

    func nodeCheckDebounceMilliseconds() -> UInt64 { 0 }

    func nodeRows(chain _: Chain, nodes: [GemNodeSelection], statuses: [String: GemNodeStatusState]) -> [GemNodeRow] {
        nodes.map {
            GemNodeRow(
                node: $0,
                title: .host(host: $0.host),
                subtitle: (statuses[$0.url] ?? .loading).subtitle(),
                latencyStatus: (statuses[$0.url] ?? .loading).latencyStatus(),
                canDelete: true,
            )
        }
    }

    func nodeStatus(chain _: Chain, url: String) async -> GemNodeStatusState {
        statusCalls.append(url)
        return statusByUrl[url] ?? .error
    }

    func nodes(chain _: Chain) async throws -> [GemNodeSelection] {
        defer { nodesCalls += 1 }
        return nodesByCall.indices.contains(nodesCalls) ? nodesByCall[nodesCalls] : nodesByCall.last ?? []
    }

    func sections() -> [GemChainSettingsSection] { [.nodes, .explorer] }

    func selectNode(chain _: Chain, url: String) async throws {
        selectedNodes.append(url)
    }

    func setExplorerName(chain _: Chain, name: String) throws {
        setExplorerNames.append(name)
    }
}

extension GemNodeSelection {
    static func mock(url: String) -> GemNodeSelection {
        GemNodeSelection(url: url, host: url, isSelected: false, gemNodeFlag: nil)
    }
}
