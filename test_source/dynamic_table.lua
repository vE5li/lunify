table = { 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, unpack({11, 12}) }
result = 1

for k, v in ipairs(table) do
    result = result + v
end

result = result - 70
