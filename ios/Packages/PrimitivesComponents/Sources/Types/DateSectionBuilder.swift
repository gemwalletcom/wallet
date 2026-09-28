// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemDayBoundaries
import GemstonePrimitives

public struct DateSectionBuilder<Item, T: Sendable & Identifiable> {
    private let items: [Item]
    private let dateKeyPath: KeyPath<Item, Date>
    private let transform: (Item) -> T

    public init(
        items: [Item],
        dateKeyPath: KeyPath<Item, Date>,
        transform: @escaping (Item) -> T,
    ) {
        self.items = items
        self.dateKeyPath = dateKeyPath
        self.transform = transform
    }

    public func build() -> [ListSection<T>] {
        let boundaries = GemDayBoundaries.current
        return boundaries.sections(days: items.map { $0[keyPath: dateKeyPath].gemDay }, newestFirst: true).map { section in
            let date = section.day.date
            return ListSection(
                id: date.ISO8601Format(),
                title: TransactionDateFormatter(date: date, boundaries: boundaries).section,
                image: nil,
                values: section.positions.map { transform(items[Int($0)]) },
            )
        }
    }
}

public extension DateSectionBuilder where Item == T {
    init(items: [Item], dateKeyPath: KeyPath<Item, Date>) {
        self.init(items: items, dateKeyPath: dateKeyPath, transform: { $0 })
    }
}
