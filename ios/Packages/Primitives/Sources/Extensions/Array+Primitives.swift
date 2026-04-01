// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension Array {
    func toMap<Key: Hashable>(_ transform: (Element) -> Key) -> [Key: Element] {
        reduce(into: [:]) { result, element in
            let key = transform(element)
            result[key] = element
        }
    }

    func toMap<Key: Hashable>(_ transform: (Element) throws -> Key) rethrows -> [Key: Element] {
        try reduce(into: [:]) { result, element in
            let key = try transform(element)
            result[key] = element
        }
    }

    func toMapArray<Key: Hashable>(_ transform: (Element) -> Key) -> [Key: [Element]] {
        reduce(into: [:]) { result, element in
            let key = transform(element)
            if let ar = result[key] {
                result[key] = ar + [element]
            } else {
                result[key] = [element]
            }
        }
    }

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

public enum ArrayError: Error {
    case indexOutOfBounds(Int)
}

public extension Array {
    func getElement(safe index: Int) throws -> Element {
        guard indices.contains(index) else {
            throw ArrayError.indexOutOfBounds(index)
        }
        return self[index]
    }

    func element(at index: Int) -> Element? {
        indices.contains(index) ? self[index] : nil
    }
}
