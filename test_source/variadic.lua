function test(first, second, ...)
    if first ~= "one" then
        return 0
    end
    if second ~= "two" then
        return 0
    end
    return arg[2]
end

result = test("one", "two", 8, 9, 10)
