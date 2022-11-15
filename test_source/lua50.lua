function increment(value)
    value = 1

    for i = 1, 15 do
        value = value + 1
    end

    return value
end

result = increment(result)

function closure()
    while result ~= 9 do
        result = result - 1
    end
end

closure()
