// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/**
 * @title BatchTransactionVerification
 * @dev Implements batch verification of pending transactions with partial failure handling
 */
contract BatchTransactionVerification {
    // Transaction structure
    struct Transaction {
        address from;
        address to;
        uint256 amount;
        bytes32 txHash;
        uint256 timestamp;
        bool verified;
        string metadata;
    }

    // Storage
    mapping(bytes32 => Transaction) public transactions;
    mapping(address => bytes32[]) public userTransactions;
    bytes32[] public pendingTransactions;
    
    // Role management
    address public owner;
    mapping(address => bool) public verifiers;

    // Events
    event TransactionSubmitted(
        bytes32 indexed txHash,
        address indexed from,
        address indexed to,
        uint256 amount,
        uint256 timestamp
    );

    event TransactionVerified(
        bytes32 indexed txHash,
        address indexed verifier,
        uint256 timestamp
    );

    event BatchVerificationCompleted(
        uint256 totalTransactions,
        uint256 successfulVerifications,
        uint256 failedVerifications,
        uint256 timestamp
    );

    event TransactionVerificationFailed(
        bytes32 indexed txHash,
        string reason,
        uint256 timestamp
    );

    event VerifierAdded(address indexed verifier, uint256 timestamp);
    event VerifierRemoved(address indexed verifier, uint256 timestamp);

    // Modifiers
    modifier onlyOwner() {
        require(msg.sender == owner, "Only owner can call this function");
        _;
    }

    modifier onlyVerifier() {
        require(verifiers[msg.sender] || msg.sender == owner, "Not authorized to verify");
        _;
    }

    constructor() {
        owner = msg.sender;
        verifiers[msg.sender] = true;
    }

    /**
     * @dev Add a new verifier
     * @param _verifier Address of the verifier to add
     */
    function addVerifier(address _verifier) external onlyOwner {
        require(_verifier != address(0), "Invalid verifier address");
        require(!verifiers[_verifier], "Verifier already exists");
        
        verifiers[_verifier] = true;
        emit VerifierAdded(_verifier, block.timestamp);
    }

    /**
     * @dev Remove a verifier
     * @param _verifier Address of the verifier to remove
     */
    function removeVerifier(address _verifier) external onlyOwner {
        require(verifiers[_verifier], "Verifier does not exist");
        require(_verifier != owner, "Cannot remove owner as verifier");
        
        verifiers[_verifier] = false;
        emit VerifierRemoved(_verifier, block.timestamp);
    }

    /**
     * @dev Submit a transaction for verification
     * @param _from Sender address
     * @param _to Recipient address
     * @param _amount Transaction amount
     * @param _metadata Additional metadata
     * @return txHash The hash of the submitted transaction
     */
    function submitTransaction(
        address _from,
        address _to,
        uint256 _amount,
        string memory _metadata
    ) external returns (bytes32) {
        require(_from != address(0), "Invalid from address");
        require(_to != address(0), "Invalid to address");
        require(_amount > 0, "Amount must be greater than 0");

        bytes32 txHash = keccak256(
            abi.encodePacked(_from, _to, _amount, block.timestamp, _metadata)
        );

        require(transactions[txHash].timestamp == 0, "Transaction already exists");

        Transaction memory newTx = Transaction({
            from: _from,
            to: _to,
            amount: _amount,
            txHash: txHash,
            timestamp: block.timestamp,
            verified: false,
            metadata: _metadata
        });

        transactions[txHash] = newTx;
        pendingTransactions.push(txHash);
        userTransactions[_from].push(txHash);

        emit TransactionSubmitted(txHash, _from, _to, _amount, block.timestamp);

        return txHash;
    }

    /**
     * @dev Verify a single transaction
     * @param _txHash Transaction hash to verify
     * @return success Whether the verification was successful
     */
    function verifySingleTransaction(bytes32 _txHash) 
        public 
        onlyVerifier 
        returns (bool) 
    {
        Transaction storage txn = transactions[_txHash];
        
        // Validate transaction exists
        if (txn.timestamp == 0) {
            emit TransactionVerificationFailed(
                _txHash, 
                "Transaction does not exist", 
                block.timestamp
            );
            return false;
        }

        // Check if already verified
        if (txn.verified) {
            emit TransactionVerificationFailed(
                _txHash, 
                "Transaction already verified", 
                block.timestamp
            );
            return false;
        }

        // Validate transaction authenticity
        if (!_validateTransactionAuthenticity(txn)) {
            emit TransactionVerificationFailed(
                _txHash, 
                "Transaction authenticity validation failed", 
                block.timestamp
            );
            return false;
        }

        // Mark as verified
        txn.verified = true;

        // Remove from pending transactions
        _removePendingTransaction(_txHash);

        emit TransactionVerified(_txHash, msg.sender, block.timestamp);

        return true;
    }

    /**
     * @dev Batch verify multiple transactions
     * @param _txHashes Array of transaction hashes to verify
     * @return results Array of booleans indicating success/failure for each transaction
     */
    function batchVerifyTransactions(bytes32[] memory _txHashes) 
        external 
        onlyVerifier 
        returns (bool[] memory) 
    {
        require(_txHashes.length > 0, "No transactions to verify");
        require(_txHashes.length <= 100, "Batch size too large (max 100)");

        bool[] memory results = new bool[](_txHashes.length);
        uint256 successCount = 0;
        uint256 failureCount = 0;

        for (uint256 i = 0; i < _txHashes.length; i++) {
            bool success = verifySingleTransaction(_txHashes[i]);
            results[i] = success;
            
            if (success) {
                successCount++;
            } else {
                failureCount++;
            }
        }

        emit BatchVerificationCompleted(
            _txHashes.length,
            successCount,
            failureCount,
            block.timestamp
        );

        return results;
    }

    /**
     * @dev Validate transaction authenticity
     * @param _txn Transaction to validate
     * @return valid Whether the transaction is authentic
     */
    function _validateTransactionAuthenticity(Transaction memory _txn) 
        private 
        view 
        returns (bool) 
    {
        // Check addresses are not zero
        if (_txn.from == address(0) || _txn.to == address(0)) {
            return false;
        }

        // Check amount is greater than 0
        if (_txn.amount == 0) {
            return false;
        }

        // Check timestamp is not in the future
        if (_txn.timestamp > block.timestamp) {
            return false;
        }

        // Check timestamp is not too old (e.g., more than 30 days)
        if (block.timestamp - _txn.timestamp > 30 days) {
            return false;
        }

        // Verify transaction hash
        bytes32 computedHash = keccak256(
            abi.encodePacked(
                _txn.from,
                _txn.to,
                _txn.amount,
                _txn.timestamp,
                _txn.metadata
            )
        );

        if (computedHash != _txn.txHash) {
            return false;
        }

        return true;
    }

    /**
     * @dev Remove a transaction from pending list
     * @param _txHash Transaction hash to remove
     */
    function _removePendingTransaction(bytes32 _txHash) private {
        for (uint256 i = 0; i < pendingTransactions.length; i++) {
            if (pendingTransactions[i] == _txHash) {
                pendingTransactions[i] = pendingTransactions[pendingTransactions.length - 1];
                pendingTransactions.pop();
                break;
            }
        }
    }

    /**
     * @dev Get all pending transactions
     * @return Array of pending transaction hashes
     */
    function getPendingTransactions() external view returns (bytes32[] memory) {
        return pendingTransactions;
    }

    /**
     * @dev Get transaction details
     * @param _txHash Transaction hash
     * @return Transaction details
     */
    function getTransaction(bytes32 _txHash) 
        external 
        view 
        returns (Transaction memory) 
    {
        return transactions[_txHash];
    }

    /**
     * @dev Get user's transactions
     * @param _user User address
     * @return Array of transaction hashes for the user
     */
    function getUserTransactions(address _user) 
        external 
        view 
        returns (bytes32[] memory) 
    {
        return userTransactions[_user];
    }

    /**
     * @dev Get pending transactions count
     * @return Number of pending transactions
     */
    function getPendingTransactionsCount() external view returns (uint256) {
        return pendingTransactions.length;
    }

    /**
     * @dev Check if a transaction is verified
     * @param _txHash Transaction hash
     * @return Whether the transaction is verified
     */
    function isTransactionVerified(bytes32 _txHash) external view returns (bool) {
        return transactions[_txHash].verified;
    }
}
