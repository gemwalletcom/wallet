// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

struct ConnectivityEndpoint: Identifiable, Sendable {
    let name: String
    let flag: String?
    let url: URL

    static let defaultEndpoints: [ConnectivityEndpoint] = [
        ConnectivityEndpoint(name: "API", flag: "🇺🇸", url: Constants.apiURL),
        ConnectivityEndpoint(name: "Nodes", flag: "🇺🇸", url: Constants.nodesURL),
        ConnectivityEndpoint(name: "Nodes", flag: "🇯🇵", url: Constants.nodesAsiaURL),
        ConnectivityEndpoint(name: "Nodes", flag: "🇪🇺", url: Constants.nodesEuropeURL),
    ]

    var id: String {
        url.absoluteString
    }

    var title: String {
        [name, flag].compactMap { $0 }.joined(separator: " ")
    }

    var host: String {
        url.host ?? url.absoluteString
    }
}
