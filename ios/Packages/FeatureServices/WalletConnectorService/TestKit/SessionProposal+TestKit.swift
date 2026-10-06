import Foundation
import WalletConnectSign

public extension Session.Proposal {
    static func mock(
        chainId: String = "tron:728126428",
        optional: Bool = false,
    ) throws -> Session.Proposal {
        let metadata: [String: Any] = ["name": "TRON dapp", "description": "", "url": "https://example.com", "icons": []]
        let namespaces: [String: Any] = ["tron": ["chains": [chainId], "methods": ["tron_signMessage"], "events": []]]
        let required = optional ? [:] : namespaces
        let optionalNamespaces = optional ? namespaces : [:]
        let proposal: [String: Any] = [
            "id": "proposal", "pairingTopic": "pairing", "proposer": metadata,
            "requiredNamespaces": required, "optionalNamespaces": optionalNamespaces,
            "proposal": [
                "relays": [], "proposer": ["publicKey": "proposer", "metadata": metadata],
                "requiredNamespaces": required, "optionalNamespaces": optionalNamespaces,
            ],
        ]
        return try JSONDecoder().decode(Session.Proposal.self, from: JSONSerialization.data(withJSONObject: proposal))
    }
}
