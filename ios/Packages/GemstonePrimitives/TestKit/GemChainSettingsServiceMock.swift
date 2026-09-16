// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone

public final class GemChainSettingsServiceMock: GemChainSettingsServiceProtocol, @unchecked Sendable {
    public var nodesByCall: [[GemNodeSelection]] = []
    public var statusByUrl: [String: GemNodeStatusState] = [:]
    public var explorerRowsValue: [GemExplorerRow] = []
    public var checkResult: Result<GemNodeCheck, GemAddNodeError> = .failure(.InvalidUrl)

    public private(set) var selectedNodes: [String] = []
    public private(set) var deletedNodes: [String] = []
    public private(set) var addedNodes: [String] = []
    public private(set) var setExplorerNames: [String] = []
    public private(set) var nodesCalls = 0
    public private(set) var statusCalls: [String] = []

    public init() {}

    public func addNode(chain _: Chain, url: String) async throws {
        addedNodes.append(url)
    }

    public func chains(query _: String) -> [Chain] { [] }

    public func checkNode(chain _: Chain, url _: String) async throws -> GemNodeCheck {
        try checkResult.get()
    }

    public func deleteNode(chain _: Chain, url: String) async throws {
        deletedNodes.append(url)
    }

    public func explorerRows(chain _: Chain) -> [GemExplorerRow] { explorerRowsValue }

    public func newAddNodeSession(chain: Chain) -> GemAddNodeSession {
        GemAddNodeSession(chain: chain, url: "", check: nil, failure: nil, isChecking: false)
    }

    public func newNodeListSession(chain: Chain) -> GemNodeListSession {
        GemNodeListSession(chain: chain, nodes: [], statuses: [:])
    }

    public func nodeCheckDebounceMilliseconds() -> UInt64 { 0 }

    public func nodeRows(chain _: Chain, nodes: [GemNodeSelection], statuses: [String: GemNodeStatusState]) -> [GemNodeRow] {
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

    public func nodeStatus(chain _: Chain, url: String) async -> GemNodeStatusState {
        statusCalls.append(url)
        return statusByUrl[url] ?? .error
    }

    public func nodes(chain _: Chain) async throws -> [GemNodeSelection] {
        defer { nodesCalls += 1 }
        return nodesByCall.indices.contains(nodesCalls) ? nodesByCall[nodesCalls] : nodesByCall.last ?? []
    }

    public func sections() -> [GemChainSettingsSection] { [.nodes, .explorer] }

    public func selectNode(chain _: Chain, url: String) async throws {
        selectedNodes.append(url)
    }

    public func setExplorerName(chain _: Chain, name: String) throws {
        setExplorerNames.append(name)
    }
}
