require 'octokit'

puts "deploy"

# Set access_token instead of login and password if you use personal access token
client = Octokit::Client.new(:access_token => ENV['GITHUB_TOKEN'])

# Fetch the current user
puts client.user
