result = 0
table = {{2}, {2}}

for v = 1, 2 do
    result = result + 1
end

for k, v in ipairs(table) do
    for _k, v in ipairs(v) do
        result = result + k + v;
    end
end
