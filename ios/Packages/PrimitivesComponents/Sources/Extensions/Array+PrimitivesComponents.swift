// Copyright (c). Gem Wallet. All rights reserved.

extension Array {
    func runs(by key: (Element) -> some Equatable) -> [[Element]] {
        reduce(into: []) { runs, element in
            if let last = runs.last?.last, key(last) == key(element) {
                runs[runs.count - 1].append(element)
            } else {
                runs.append([element])
            }
        }
    }
}
