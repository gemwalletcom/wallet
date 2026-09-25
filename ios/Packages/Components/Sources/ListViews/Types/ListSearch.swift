// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Observation

@Observable
public final class ListSearch<Item> {
    public let filter: ([Item], String) -> [Item]
    public let emptyContent: any EmptyContentViewable
    public var query = ""

    public init(
        filter: @escaping ([Item], String) -> [Item],
        emptyContent: any EmptyContentViewable,
    ) {
        self.filter = filter
        self.emptyContent = emptyContent
    }
}

extension ListSearch where Item: Identifiable & Sendable {
    func filtered(_ type: SelectableListType<Item>) -> SelectableListType<Item> {
        guard !query.isEmpty else { return type }
        return switch type {
        case let .plain(items):
            .plain(filter(items, query))
        case let .section(sections):
            .section(
                sections
                    .map { ListSection(id: $0.id, title: $0.title, image: $0.image, values: filter($0.values, query)) }
                    .filter { !$0.values.isEmpty },
            )
        }
    }
}
