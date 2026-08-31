// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension Array {
    func splitInSubArrays(into size: Int) -> [[Element]] {
        (0 ..< size).map {
            stride(from: $0, to: count, by: size).map { self[$0] }
        }
    }

    func chunks(_ chunkSize: Int) -> [[Element]] {
        stride(from: 0, to: count, by: chunkSize).map {
            Array(self[$0 ..< Swift.min($0 + chunkSize, self.count)])
        }
    }

    func shuffleInGroups(groupSize: Int) -> [Element] {
        let groups = stride(from: 0, to: count, by: groupSize)
            .map { Array(self[$0 ..< Swift.min($0 + groupSize, count)]) }
        return groups.map { $0.shuffled() }.flatMap(\.self)
    }
}

public extension Array where Element: Hashable {
    func distinct() -> [Element] {
        Array(Set(self))
    }

    func asSet() -> Set<Element> {
        Set(self)
    }

    subscript(safe index: Index) -> Element? {
        indices.contains(index) ? self[index] : nil
    }
}

public extension Sequence where Iterator.Element: Hashable {
    func unique() -> [Iterator.Element] {
        var elements = Set<Iterator.Element>()
        return filter { elements.insert($0).inserted }
    }
}

public extension Array {
    func element(at index: Int) -> Element? {
        indices.contains(index) ? self[index] : nil
    }
}
