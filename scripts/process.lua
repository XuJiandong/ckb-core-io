-- Check for command line argument
if #arg ~= 1 then
    print("Usage: lua process.lua <folder>")
    os.exit(1)
end

-- Read the contents of the file
local function read_file(file)
    local f = io.open(file, "r")
    if not f then
        print("Could not open file: " .. file)
        os.exit(1)
    end
    local content = f:read("*a")
    f:close()
    return content
end

-- Write the modified contents back to the file
local function write_file(file, content)
    local f = io.open(file, "w")
    if not f then
        print("Could not write to file: " .. file)
        os.exit(1)
    end
    f:write(content)
    f:close()
end

-- Process the content to remove patterns
local function process_content(content)
    -- Remove #[stable(...)] and #[unstable(...)]
    content = content:gsub("#%!%[stable%s*%(%s*feature%s*=%s*\"[^\"]+\"%s*,?%s*[^%)]*%)]", "")
    content = content:gsub("#%[stable%s*%(%s*feature%s*=%s*\"[^\"]+\"%s*,?%s*[^%)]*%)]", "")
    content = content:gsub("#%[unstable%s*%(%s*feature%s*=%s*\"[^\"]+\"%s*,?%s*[^%)]*%)]", "")
    content = content:gsub("#%[rustc_const_stable%s*%(%s*feature%s*=%s*\"[^\"]+\"%s*,?%s*[^%)]*%)]", "")
    content = content:gsub("crate::cmp", "core::cmp")
    content = content:gsub("crate::fmt", "alloc::fmt")
    content = content:gsub("crate::mem", "core::mem")
    content = content:gsub("crate::ops", "core::ops")
    content = content:gsub("crate::slice", "core::slice")
    content = content:gsub("crate::str", "alloc::str")
    content = content:gsub("crate::alloc", "alloc")
    content = content:gsub("crate::collections", "alloc::collections")
    content = content:gsub("crate::io", "crate")
    content = content:gsub("default fn", "fn")
    content = content:gsub("crate::sys_common::io::DEFAULT_BUF_SIZE", "1024")
    content = content:gsub("io::Result", "crate::Result")
    content = content:gsub("io::Error", "crate::Error")
    return content
end


local function main(file)
    local content = read_file(file)
    local modified_content = process_content(content)
    write_file(file, modified_content)

    print("Processing completed:" .. file)
end

local file = arg[1]
main(file)
